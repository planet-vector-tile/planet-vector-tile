use super::{
    hilbert_tile::{Chunk, HilbertTile},
    leaf::Leaf,
};
use crate::{
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{Node, Osm, Relation, Way},
};
use std::io::BufWriter;
use std::{fs::File, ops::Range, path::Path};

pub fn build_chunks(
    m_leaves: &Mutant<Leaf>,
    m_tiles: &Mutant<HilbertTile>,
    m_leaves_external: &Mutant<u32>,
    dir: &Path,
    archive: &Osm,
    leaf_zoom: u8,
) -> Result<(Mutant<Chunk>, Mutant<Chunk>, Mutant<Chunk>), Box<dyn std::error::Error>> {
    let leaves = m_leaves.slice();
    let tiles = m_tiles.slice();
    let tiles_mut = m_tiles.mutable_slice();
    let nodes = archive.nodes();
    let ways = archive.ways();
    let external = m_leaves_external.slice();

    let n_chunks_writer = Mutant::<u64>::empty_buffered_writer(dir, "hilbert_n_chunks")?;
    let w_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_w_chunks")?;
    let r_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_r_chunks")?;
    let mut n_chunk_count = 0;
    let mut w_chunk_count = 0;
    let mut r_chunk_count = 0;

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;

    for i in 0..tiles.len() {
        let tile = &tiles[i];
        let next_tile = if i + 1 < tiles.len() {
            Some(&tiles[i + 1])
        } else {
            None
        };

        children += count_children(tile.mask);

        println!(
            "z {} children {} total_children {} {:?}",
            z, children, total_children, tile
        );

        // Get a vec of references to all of the entities in the tile.
        // We will then filter from this to build chunks.
        let (nodes, ways) = if z == leaf_zoom - 2 {
            let first_leaf = &leaves[tile.child as usize];
            // The leaf after the last leaf of this tile's children. (the first leaf of the next tile)
            let end_leaf = match next_tile {
                Some(next_tile) => Some(&leaves[next_tile.child as usize]),
                None => None,
            };

            let nodes = match end_leaf {
                Some(end_leaf) => {
                    if end_leaf.n == 0 {
                        println!("end leaf n == 0");
                    }
                    &nodes[first_leaf.n as usize..end_leaf.n as usize]
                },
                None => &nodes[first_leaf.n as usize..],
            };

            let ways = match end_leaf {
                Some(end_leaf) => {
                    let inner_ways = ways[first_leaf.w as usize..end_leaf.w as usize].iter();
                    let ext_ways = external[first_leaf.w_ext as usize..end_leaf.w_ext as usize]
                        .iter()
                        .map(|i| &ways[*i as usize]);
                    let all_ways = inner_ways.chain(ext_ways);
                    all_ways.collect::<Vec<&Way>>()
                }
                None => {
                    let inner_ways = ways[first_leaf.w as usize..]
                        .iter()
                        .map(|w| w); // this peels out the reference to the way without removing the way
                    let ext_ways = external[first_leaf.w_ext as usize..]
                        .iter()
                        .map(|i| &ways[*i as usize]);
                    let all_ways = inner_ways.chain(ext_ways);
                    all_ways.collect::<Vec<&Way>>()
                }
            };

            (nodes, ways)
        } else {
            (&nodes[0..0], vec![])
        };

        println!("nodes {} ways {}", nodes.len(), ways.len());

        level_tile_count += 1;
        // If we are done with the level, decrement z to the next zoom.
        if children >= total_children && z > 0 {
            total_children = level_tile_count;
            level_tile_count = 0;
            children = 0;
            z -= 2;
        }
    }

    let n_chunks = Mutant::<Chunk>::open(dir, "hilbert_node_chunks", false)?;
    let w_chunks = Mutant::<Chunk>::open(dir, "hilbert_way_chunks", false)?;
    let r_chunks = Mutant::<Chunk>::open(dir, "hilbert_relation_chunks", false)?;

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

fn chunk_nodes(
    range: Range<usize>,
    nodes: &[Node],
    writer: &BufWriter<File>,
    count: &mut u32,
    zoom: u8,
) {
    for i in range {
        let node = &nodes[i];

        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use flatdata::FileResourceStorage;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_chunks() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let archive = Osm::open(FileResourceStorage::new(&dir)).unwrap();
        let m_leaves = Mutant::<Leaf>::open(&dir, "hilbert_leaves", false).unwrap();
        let m_tiles = Mutant::<HilbertTile>::open(&dir, "hilbert_tiles", false).unwrap();
        let m_leaves_external = Mutant::<u32>::open(&dir, "hilbert_leaves_external", false).unwrap();
        let _ = build_chunks(&m_leaves, &m_tiles, &m_leaves_external, &dir, &archive, 12);
    }
}
