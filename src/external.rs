use std::path::Path;

use dashmap::mapref::entry::Entry::{Occupied, Vacant};
use dashmap::DashMap;
use std::collections::BTreeSet;
use rayon::prelude::*;

use crate::{
    hilbert::HilbertTree,
    location::h_to_zoom_h,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair},
};

pub fn populate_hilbert_leaves_external(
    tree: &HilbertTree,
    dir: &Path,
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
) -> Result<Mutant<u32>, Box<dyn std::error::Error>> {
    let mut leaf_to_ways: DashMap<u32, BTreeSet<u32>> = DashMap::new();

    let ways = tree.archive.ways();
    let way_pairs = m_way_pairs.slice();
    let node_pairs = m_node_pairs.slice();
    let nodes_index = tree.archive.nodes_index();
    let nodes_index_len = nodes_index.len();

    ways.par_iter().enumerate().for_each(|(i, way)| {
        let way_h = way_pairs[i].h();
        let way_tile_h = h_to_zoom_h(way_h, tree.leaf_zoom) as u32;

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
                let tile_h = h_to_zoom_h(h, tree.leaf_zoom) as u32;
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
    println!("{:?}", leaf_to_ways);
    let m_leaves_external = Mutant::<u32>::new(dir, "hilbert_leaves_external", 10)?;
    Ok(m_leaves_external)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_basic() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let tree = HilbertTree::open(&dir, 12).unwrap();
        let m_node_pairs = Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let ext = populate_hilbert_leaves_external(&tree, &dir, &m_node_pairs, &m_way_pairs).unwrap();
    }
}
