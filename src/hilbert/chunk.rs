use super::{
    hilbert_tile::{Chunk, HilbertTile},
    leaf::Leaf,
};
use crate::{
    filter::Filter,
    manifest::Manifest,
    mutant::{Mutant, to_bytes},
    osmflat::osmflat_generated::osm::{Node, Osm, Way}, location,
};
use std::{path::Path, io::{BufWriter, Write}, fs::File};

type Err = Box<dyn std::error::Error>;

static mut ONE:u32 = 0;

pub fn build_chunks(
    m_leaves: &Mutant<Leaf>,
    m_tiles: &Mutant<HilbertTile>,
    m_leaves_external: &Mutant<u32>,
    dir: &Path,
    archive: &Osm,
    manifest: &Manifest,
) -> Result<(Mutant<Chunk>, Mutant<Chunk>, Mutant<Chunk>), Err> {
    let filter = Filter::new(manifest, archive);
    let leaf_zoom = manifest.render.leaf_zoom;
    let leaves = m_leaves.slice();
    let tiles = m_tiles.slice();
    let tiles_mut = m_tiles.mutable_slice();
    let nodes = archive.nodes();
    let ways = archive.ways();
    let external = m_leaves_external.slice();

    // Delete previous chunks if they exist.
    let _ = std::fs::remove_file(dir.join("hilbert_n_chunks"));
    let _ = std::fs::remove_file(dir.join("hilbert_w_chunks"));

    let mut n_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_n_chunks")?;
    let mut w_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_w_chunks")?;
    let mut n_chunk_count = 0;
    let mut w_chunk_count = 0;

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;

    for i in 0..tiles.len() {
        let tile = &tiles[i];

        children += count_children(tile.mask);

        // some if it is not the last tile or the last tile of the level
        let next_tile = if i + 1 < tiles.len() && children < total_children {
            Some(&tiles[i + 1])
        } else {
            None
        };

        let node_filter = filter.node_at_zoom(z);
        let way_filter = filter.way_at_zoom(z);

        // Get a vec of indices to all of the entities in the tile.
        // We will then filter from this to build chunks.
        let (nodes, ways): (Vec<usize>, Vec<usize>) = if z == leaf_zoom - 2 {
            let first_leaf = &leaves[tile.child as usize];
            // The leaf after the last leaf of this tile's children. (the first leaf of the next tile)
            let end_leaf = match next_tile {
                Some(next_tile) => Some(&leaves[next_tile.child as usize]),
                None => None,
            };

            let nodes_range = match end_leaf {
                Some(end_leaf) => first_leaf.n as usize..end_leaf.n as usize,
                None => first_leaf.n as usize..nodes.len(),
            };
            let nodes = nodes_range.map(|i| (i, &nodes[i]));
            let filtered_nodes: Vec<usize> = nodes.filter(node_filter).map(|(i, _)| i).collect();

            let w_range = match end_leaf {
                Some(end_leaf) => first_leaf.w as usize..end_leaf.w as usize,
                None => first_leaf.w as usize..ways.len(),
            };
            let w_ext_range = match end_leaf {
                Some(end_leaf) => first_leaf.w_ext as usize..end_leaf.w_ext as usize,
                None => first_leaf.w_ext as usize..external.len(),
            };

            let inner_ways = w_range.map(|i| (i, &ways[i]));
            let ext_ways = external[w_ext_range].iter().map(|&i| (i as usize, &ways[i as usize]));
            let ways = inner_ways.chain(ext_ways);
            let filtered_ways: Vec<usize> = ways.filter(way_filter).map(|(i, _)| i).collect();

            (filtered_nodes, filtered_ways)
        } else {
            (vec![], vec![])
        };

        let origin_leaf = get_origin_leaf(i, z, leaf_zoom, tiles, leaves);
        let origin_n_idx = origin_leaf.n as usize;
        let origin_w_idx = origin_leaf.w as usize;

        let n_start = n_chunk_count;
        let w_start = w_chunk_count;
        if write_chunks(&nodes, origin_n_idx, &mut n_chunks_writer, &mut n_chunk_count)? {
            tiles_mut[i].n_chunk = n_start;
        };
        if write_chunks(&ways, origin_w_idx, &mut w_chunks_writer, &mut w_chunk_count)? {
            tiles_mut[i].w_chunk = w_start;
        };

        println!(
            "z {} children {} total_children {} nodes {} ways {} {:?}",
            z,
            children,
            total_children,
            nodes.len(),
            ways.len(),
            tile
        );

        level_tile_count += 1;
        // If we are done with the level, decrement z to the next zoom.
        if children >= total_children && z > 0 {
            total_children = level_tile_count;
            level_tile_count = 0;
            children = 0;
            z -= 2;
        }
    }

    let n_chunks = Mutant::<Chunk>::open(dir, "hilbert_n_chunks", false)?;
    let w_chunks = Mutant::<Chunk>::open(dir, "hilbert_w_chunks", false)?;
    let r_chunks = Mutant::<Chunk>::new(dir, "hilbert_r_chunks", 0)?;

    Ok((n_chunks, w_chunks, r_chunks))
}

fn count_children(mask: u16) -> u32 {
    let mut count = 0;
    for i in 0..16 {
        if mask >> i & 1 == 1 {
            count += 1;
        }
    }
    count
}

fn get_origin_leaf<'a>(tiles_i: usize, zoom: u8, leaf_zoom: u8, tiles: &[HilbertTile], leaves: &'a [Leaf]) -> &'a Leaf {
    let mut tile = &tiles[tiles_i];
    let mut z = zoom;
    while z < leaf_zoom - 2 {
        tile = &tiles[tile.child as usize];
        z += 2;
    }
    &leaves[tile.child as usize]
}

fn write_chunks(entities: &Vec<usize>, origin_idx: usize, writer: &mut BufWriter<File>, chunk_count: &mut u32) -> Result<bool, Err> {
    if let Some(first) = entities.first() {
        let mut chunk = Chunk {
            offset: (first - origin_idx) as i32,
            length: 1,
        };
        let mut prev_idx = *first;
        for &idx in entities[1..].iter() {
            if idx == prev_idx + 1 {
                chunk.length += 1;
            } else {
                let bytes = unsafe { to_bytes(&chunk) };
                writer.write(bytes)?;
                *chunk_count += 1;
                chunk = Chunk {
                    offset: (idx - origin_idx) as i32,
                    length: 1,
                };
            }
            prev_idx = idx;
        }
        let bytes = unsafe { to_bytes(&chunk) };
        writer.write_all(bytes)?;
        *chunk_count += 1;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use flatdata::FileResourceStorage;

    use crate::manifest;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_chunks() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let archive = Osm::open(FileResourceStorage::new(&dir)).unwrap();
        let m_leaves = Mutant::<Leaf>::open(&dir, "hilbert_leaves", false).unwrap();
        let m_tiles = Mutant::<HilbertTile>::open(&dir, "hilbert_tiles", false).unwrap();
        let m_leaves_external =
            Mutant::<u32>::open(&dir, "hilbert_leaves_external", false).unwrap();
        let _ = build_chunks(
            &m_leaves,
            &m_tiles,
            &m_leaves_external,
            &dir,
            &archive,
            &manifest::parse(None),
        );
    }
}
