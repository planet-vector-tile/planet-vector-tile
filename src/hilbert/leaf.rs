use std::io::{Error, ErrorKind};
use std::path::Path;
use std::time::Instant;

use crate::location;
use crate::osmflat::osmflat_generated::osm::Osm;
use crate::tile::tile_count_for_zoom;
use crate::{
    location::h_to_zoom_h,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair},
};
use dashmap::mapref::entry::Entry::{Occupied, Vacant};
use dashmap::DashMap;
use humantime::format_duration;
use log::info;
use rayon::prelude::*;
use std::collections::BTreeSet;

// Leaves correspond to additional info we need to know about the tiles at the leaf level.
// We need to know:
//  - The indices into the nodes, ways, relations vectors.
//  - The hilbert index that the given tile starts at.
// Though the hilbert index can be derived from the n,w,r by looking at the hilbert pairs,
// This is referenced often, so this is simpler and saves us from paging into the entity
// (nodes, ways, relations) vectors unnecessarily.
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Leaf {
    // Indices to the first node of the given leaf tile.
    pub n: u64,
    pub w: u32,
    pub r: u32,
    // Hilbert index for the leaf tile, at the leaf zoom
    pub h: u32,
    // Indices to the first of ways in relations in w_ext and r_ext
    // denoting ways and relations that enter the given leaf tile
    // that exist outside of the leaf's n,w,r ranges.
    pub w_ext: u32,
    pub r_ext: u32,
}

pub fn build_leaves(
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<Mutant<Leaf>, Box<dyn std::error::Error>> {
    let node_pairs = m_node_pairs.slice();
    let way_pairs = m_way_pairs.slice();

    if node_pairs.len() == 0 && way_pairs.len() == 0 {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            "No hilbert pairs found! Cannot build hilbert tiles.",
        )));
    }

    let mut n_i: usize = 0; // node hilbert pair index
    let mut w_i: usize = 0; // way hilbert pair index

    let mut lowest_h = 0;

    // Find the lowest hilbert tile
    if let Some(first_node_pair) = node_pairs.first() {
        lowest_h = first_node_pair.h();
        n_i = 1;
    }
    if let Some(first_way_pair) = way_pairs.first() {
        let first_way_h = first_way_pair.h();
        if first_way_h < lowest_h {
            lowest_h = first_way_h;
            w_i = 1;
            n_i = 0;
        }
    }

    // First leaf Hilbert tile has the lowest hilbert location.
    let mut tile_h = location::h_to_zoom_h(lowest_h, leaf_zoom) as u32;
    info!(
        "Lowest tile_h for leaves in hilbert tree: {}, leaf_zoom: {}",
        lowest_h, leaf_zoom
    );

    // NHTODO Implement the ability to grow the LeafTile mutant so that we don't have to allocate max size upfront?
    let max_len = tile_count_for_zoom(leaf_zoom) as usize;
    let mut m_leaves = Mutant::<Leaf>::new(dir, "hilbert_leaves", max_len)?;
    let leaves = m_leaves.mutable_slice();

    let node_pairs = m_node_pairs.slice();
    let node_pairs_len = node_pairs.len();
    let way_pairs = m_way_pairs.slice();
    let way_pairs_len = way_pairs.len();

    println!("LEAVES zoom {}", leaf_zoom);

    // First leaf tile
    let first_leaf = Leaf {
        n: 0,
        w: 0,
        r: 0,
        h: tile_h,
        w_ext: 0,
        r_ext: 0,
    };
    println!("0 {:?}", first_leaf);
    leaves[0] = first_leaf;

    let mut leaf_i = 1;

    let mut next_node_tile_h = tile_h;
    let mut next_way_tile_h = tile_h;

    loop {
        let mut node_changed = false;
        while n_i < node_pairs_len && next_node_tile_h <= tile_h {
            let node_h = node_pairs[n_i].h();
            let node_tile_h = location::h_to_zoom_h(node_h, leaf_zoom) as u32;
            if node_tile_h > tile_h {
                next_node_tile_h = node_tile_h;
                node_changed = true;
                break;
            }
            n_i += 1;
        }

        let mut way_changed = false;
        while w_i < way_pairs_len && next_way_tile_h <= tile_h {
            let way_h = way_pairs[w_i].h();
            let way_tile_h = location::h_to_zoom_h(way_h, leaf_zoom) as u32;
            if way_tile_h > tile_h {
                next_way_tile_h = way_tile_h;
                way_changed = true;
                break;
            }
            w_i += 1;
        }

        let next_tile_h = if !way_changed || (node_changed && next_node_tile_h < next_way_tile_h) {
            next_node_tile_h
        } else {
            next_way_tile_h
        };

        if next_tile_h > tile_h {
            let leaf = Leaf {
                n: n_i as u64,
                w: w_i as u32,
                r: 0,
                h: next_tile_h,
                w_ext: 0,
                r_ext: 0,
            };
            println!("{} {:?}", leaf_i, leaf);
            leaves[leaf_i] = leaf;
            tile_h = next_tile_h;
            leaf_i += 1;
        } else {
            break;
        }
    }

    // The last increment of t_i falls through both whiles, so it is equal to the length.
    m_leaves.set_len(leaf_i);
    m_leaves.trim();
    Ok(m_leaves)
}

pub fn populate_hilbert_leaves_external(
    dir: &Path,
    archive: &Osm,
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
    m_leaves: &Mutant<Leaf>,
    leaf_zoom: u8,
) -> Result<Mutant<u32>, Box<dyn std::error::Error>> {
    // NHTODO Profile memory usage here.
    let leaf_to_ways: DashMap<u32, BTreeSet<u32>> = DashMap::new();

    let ways = archive.ways();
    let way_pairs = m_way_pairs.slice();
    let node_pairs = m_node_pairs.slice();
    let nodes_index = archive.nodes_index();
    let nodes_index_len = nodes_index.len();

    let t = Instant::now();
    println!("Populating external leaf entities...");

    ways.par_iter().enumerate().for_each(|(i, way)| {
        let way_h = way_pairs[i].h();
        let way_tile_h = h_to_zoom_h(way_h, leaf_zoom) as u32;

        let refs = way.refs();
        let start = refs.start as usize;
        let end = if refs.end == 0 {
            nodes_index_len
        } else {
            refs.end as usize
        };

        for n in &nodes_index[start..end] {
            if let Some(v) = n.value() {
                let h = node_pairs[v as usize].h();
                let tile_h = h_to_zoom_h(h, leaf_zoom) as u32;
                if tile_h != way_tile_h {
                    match leaf_to_ways.entry(tile_h) {
                        Occupied(mut o) => {
                            o.get_mut().insert(i as u32);
                        }
                        Vacant(v) => {
                            v.insert(BTreeSet::from([i as u32]));
                        }
                    }
                }
            }
        }
    });

    let mut leaves_ext = Mutant::<u32>::with_capacity(dir, "hilbert_leaves_external", 1024)?;

    let leaves = m_leaves.mutable_slice();

    let mut counter: u32 = 0;

    for i in 0..leaves.len() {
        let leaf = &mut leaves[i];
        let h = leaf.h;
        if let Some(ways) = leaf_to_ways.get(&h) {
            let mut it = ways.iter();
            let Some(&first) = it.next() else { break; };
            leaf.w_ext = counter;
            counter += 1;
            leaves_ext.push(first);
            for &way_i in it {
                leaves_ext.push(way_i);
            }
        } else {
            leaf.w_ext = counter;
        }
    }

    println!(
        "Populated external leaf entities in {}",
        format_duration(t.elapsed())
    );

    leaves_ext.trim();
    Ok(leaves_ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet;
    use flatdata::FileResourceStorage;
    use std::path::PathBuf;

    #[test]
    fn test_populate_hilbert_leaves_external() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let archive = Osm::open(FileResourceStorage::new(&dir)).unwrap();
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();
        let m_leaves = Mutant::<Leaf>::open(&dir, "hilbert_leaves", false).unwrap();

        let m_ext = populate_hilbert_leaves_external(
            &dir,
            &archive,
            &m_node_pairs,
            &m_way_pairs,
            &m_leaves,
            12,
        )
        .unwrap();
        let ext = m_ext.slice();
        assert_eq!(ext.len(), 4633);

        let mut osm_id = 0;
        let mut count = 0;
        let ways = archive.ways();
        for i in 0..ext.len() {
            let w = &ways[i];
            osm_id = w.osm_id();
            count += 1;
        }
        assert_eq!(count, ext.len());
        assert_eq!(osm_id, 962148640);

        // Check that w_ext is ascending or equal for the leaves.
        let mut leaves_it = m_leaves.slice().iter();
        let mut leaf = leaves_it.next().unwrap();
        let mut next = leaves_it.next();
        while next.is_some() {
            let next_leaf = next.unwrap();
            assert!(leaf.w_ext <= next_leaf.w_ext);
            leaf = next_leaf;
            next = leaves_it.next();
        }
    }

    #[test]
    fn test_4nodes_leaf_tiles() {
        let dir = PathBuf::from("./tests/fixtures/nodes4/sort");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let node_pairs = m_node_pairs.slice();
        let mut leaf_tiles = AHashSet::<u32>::new();
        for p in node_pairs {
            let zoom_h = location::h_to_zoom_h(p.h(), 12) as u32;
            leaf_tiles.insert(zoom_h);
            // println!("{:?} zoom_h: {}", p, zoom_h);
        }
        // Should be a total of 3 unique tiles for the 4 nodes.
        assert_eq!(leaf_tiles.len(), 3);
    }

    #[test]
    fn test_build_leaves() {
        let dir = PathBuf::from("./tests/fixtures/nodes4/sort");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, 12).unwrap();

        // We know there are 3 unique leaf tiles for the 4 nodes.
        assert_eq!(m_leaves.len, 3);
    }
}
