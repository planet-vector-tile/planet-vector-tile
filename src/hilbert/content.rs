use super::{hilbert_tile::HilbertTile, leaf::Leaf};
use crate::{
    filter::Filter, manifest::Manifest, mutant::Mutant, osmflat::osmflat_generated::osm::Osm,
};
use std::{io::Write, path::Path};

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

    let mut n_writer = Mutant::<u64>::empty_buffered_writer(dir, "n")?;
    let mut w_writer = Mutant::<u32>::empty_buffered_writer(dir, "w")?;
    let mut n_count: usize = 0;
    let mut w_count: usize = 0;

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
        // We will then filter from this to populate tile content.
        let (nodes, ways): (Vec<u64>, Vec<u32>) = if z == leaf_zoom - 2 {
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
            let filtered_nodes: Vec<u64> =
                nodes.filter(node_filter).map(|(i, _)| i as u64).collect();

            let w_range = match end_leaf {
                Some(end_leaf) => first_leaf.w as usize..end_leaf.w as usize,
                None => first_leaf.w as usize..ways.len(),
            };
            let w_ext_range = match end_leaf {
                Some(end_leaf) => first_leaf.w_ext as usize..end_leaf.w_ext as usize,
                None => first_leaf.w_ext as usize..external.len(),
            };

            let inner_ways = w_range.map(|i| (i, &ways[i]));
            let ext_ways = external[w_ext_range]
                .iter()
                .map(|&i| (i as usize, &ways[i as usize]));
            let ways = inner_ways.chain(ext_ways);
            let filtered_ways: Vec<u32> = ways.filter(way_filter).map(|(i, _)| i as u32).collect();

            (filtered_nodes, filtered_ways)
        } else {
            (vec![], vec![])
        };

        tiles_mut[i].n = n_count as u64;
        tiles_mut[i].w = w_count as u32;

        // Yeehaw!!!
        let n_bytes = unsafe { to_bytes(&nodes) };
        let w_bytes = unsafe { to_bytes(&ways) };

        n_writer.write(&n_bytes)?;
        w_writer.write(&w_bytes)?;

        n_count += nodes.len();
        w_count += ways.len();

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

    n_writer.flush()?;
    w_writer.flush()?;

    let mut n = Mutant::<u64>::open(dir, "n", false)?;
    let mut w = Mutant::<u32>::open(dir, "w", false)?;
    let r = Mutant::<u32>::new(dir, "r", 0)?;

    n.set_len(n_count);
    w.set_len(w_count);

    Ok((n, w, r))
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
