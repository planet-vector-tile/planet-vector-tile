use humantime::format_duration;

use super::{hilbert_tile::HilbertTile, leaf::Leaf};
use crate::{
    filter::Filter, manifest::Manifest, mutant::Mutant, osmflat::osmflat_generated::osm::Osm,
};
use std::{path::Path, time::Instant};

type Err = Box<dyn std::error::Error>;

static mut ONE: u32 = 0;

pub fn populate_tile_content(
    m_leaves: &Mutant<Leaf>,
    m_tiles: &Mutant<HilbertTile>,
    m_leaves_external: &Mutant<u32>,
    dir: &Path,
    archive: &Osm,
    manifest: &Manifest,
) -> Result<(Mutant<u64>, Mutant<u32>, Mutant<u32>), Err> {
    let filter = Filter::new(manifest, archive);
    let leaf_zoom = manifest.render.leaf_zoom;
    let leaves = m_leaves.slice();
    let tiles = m_tiles.slice();
    let tiles_mut = m_tiles.mutable_slice();
    let nodes = archive.nodes();
    let ways = archive.ways();
    let external = m_leaves_external.slice();

    // Delete previous contents
    let _ = std::fs::remove_file(dir.join("n"));
    let _ = std::fs::remove_file(dir.join("w"));
    let _ = std::fs::remove_file(dir.join("r"));

    let mut m_n = Mutant::<u64>::with_capacity(dir, "n", 1024)?;
    let mut m_w = Mutant::<u32>::with_capacity(dir, "w", 1024)?;
    let mut m_r = Mutant::<u32>::with_capacity(dir, "r", 1024)?;

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;

    let t = Instant::now();
    println!("Populating tile content...");

    for i in 0..tiles.len() {
        let tile = &tiles[i];

        children += count_children(tile.mask);

        let node_filter = filter.node_at_zoom(z);
        let way_filter = filter.way_at_zoom(z);

        // Get a vec of indices to all of the entities in the tile.
        // We will then filter from this to populate tile content.
        let (nodes, ways): (Vec<u64>, Vec<u32>) = if z == leaf_zoom - 2 {
            let next_tile = if i + 1 < tiles.len() && children < total_children {
                Some(&tiles[i + 1])
            } else {
                None
            };

            let start_leaf = &leaves[tile.child as usize];
            // The leaf after the last leaf of this tile's children. (the first leaf of the next tile)
            let end_leaf = match next_tile {
                Some(next_tile) => Some(&leaves[next_tile.child as usize]),
                None => None,
            };

            let nodes_range = match end_leaf {
                Some(end_leaf) => start_leaf.n as usize..end_leaf.n as usize,
                None => start_leaf.n as usize..nodes.len(),
            };
            let nodes = nodes_range.map(|i| (i, &nodes[i]));
            let filtered_nodes: Vec<u64> =
                nodes.filter(node_filter).map(|(i, _)| i as u64).collect();

            let w_range = match end_leaf {
                Some(end_leaf) => start_leaf.w as usize..end_leaf.w as usize,
                None => start_leaf.w as usize..ways.len(),
            };
            let w_ext_range = match end_leaf {
                Some(end_leaf) => start_leaf.w_ext as usize..end_leaf.w_ext as usize,
                None => start_leaf.w_ext as usize..external.len(),
            };

            let inner_ways = w_range.map(|i| (i, &ways[i]));
            let ext_ways = external[w_ext_range]
                .iter()
                .map(|&i| (i as usize, &ways[i as usize]));
            let ways = inner_ways.chain(ext_ways);
            let filtered_ways: Vec<u32> = ways.filter(way_filter).map(|(i, _)| i as u32).collect();

            (filtered_nodes, filtered_ways)
        } else {
            let next_tile = if i + 1 < tiles.len() {
                Some(&tiles[i + 1])
            } else {
                None
            };

            let start_child = &tiles[tile.child as usize];
            let end_child = match next_tile {
                Some(next_tile) => Some(&tiles[next_tile.child as usize]),
                None => None,
            };

            let n_idxs = m_n.slice();

            let node_idxs = match end_child {
                Some(end_child) => &n_idxs[start_child.n as usize..end_child.n as usize],
                None => &n_idxs[start_child.n as usize..],
            };

            let nodes = node_idxs.iter().map(|i| (*i as usize, &nodes[*i as usize]));
            let filtered_nodes: Vec<u64> =
                nodes.filter(node_filter).map(|(i, _)| i as u64).collect();

            let w_idxs = m_w.slice();

            let way_idxs = match end_child {
                Some(end_child) => &w_idxs[start_child.w as usize..end_child.w as usize],
                None => &w_idxs[start_child.w as usize..],
            };

            let ways = way_idxs.iter().map(|i| (*i as usize, &ways[*i as usize]));
            let filtered_ways: Vec<u32> = ways.filter(way_filter).map(|(i, _)| i as u32).collect();

            (filtered_nodes, filtered_ways)
        };

        tiles_mut[i].n = m_n.len as u64;
        tiles_mut[i].w = m_w.len as u32;

        m_n.append(&nodes)?;
        m_w.append(&ways)?;

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

    m_n.trim();
    m_w.trim();
    m_r.trim();

    println!(
        "Populating tile content took {}",
        format_duration(t.elapsed())
    );

    Ok((m_n, m_w, m_r))
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

fn get_origin_leaf<'a>(
    tiles_i: usize,
    zoom: u8,
    leaf_zoom: u8,
    tiles: &[HilbertTile],
    leaves: &'a [Leaf],
) -> &'a Leaf {
    let mut tile = &tiles[tiles_i];
    let mut z = zoom;
    while z < leaf_zoom - 2 {
        tile = &tiles[tile.child as usize];
        z += 2;
    }
    &leaves[tile.child as usize]
}

pub unsafe fn to_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
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
        let _ = populate_tile_content(
            &m_leaves,
            &m_tiles,
            &m_leaves_external,
            &dir,
            &archive,
            &manifest::parse(None),
        );
    }
}
