use std::io::{BufWriter, Write};
use std::path::Path;

use dashmap::mapref::entry::Entry::{Occupied, Vacant};
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::BTreeSet;

use crate::hilbert::Leaf;
use crate::osmflat::osmflat_generated::osm::Osm;
use crate::{
    location::h_to_zoom_h,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair},
};

pub fn populate_hilbert_leaves_external(
    dir: &Path,
    archive: &Osm,
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
    m_leaves: &Mutant<Leaf>,
    leaf_zoom: u8,
) -> Result<Mutant<u32>, Box<dyn std::error::Error>> {
    let leaf_to_ways: DashMap<u32, BTreeSet<u32>> = DashMap::new();

    let ways = archive.ways();
    let way_pairs = m_way_pairs.slice();
    let node_pairs = m_node_pairs.slice();
    let nodes_index = archive.nodes_index();
    let nodes_index_len = nodes_index.len();

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

    let leaves_ext_file = Mutant::<u32>::empty_file(dir, "hilbert_leaves_external")?;
    let mut leaves_ext_stream = BufWriter::new(&leaves_ext_file);
    let mut leaves_ext_count = 0;

    let leaves = m_leaves.mutable_slice();

    for i in 0..leaves.len() {
        let leaf = &mut leaves[i];
        if let Some(ways) = leaf_to_ways.get(&leaf.h) {
            leaf.w_ext = leaves_ext_count;
            leaves_ext_count += 1;
            for &way_i in ways.iter() {
                leaves_ext_stream.write_all(&way_i.to_le_bytes())?;
            }
        }
    }

    let mut m_leaves_external = Mutant::<u32>::open(dir, "hilbert_leaves_external", false)?;
    m_leaves_external.set_len(leaves_ext_count as usize);
    Ok(m_leaves_external)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::hilbert::HilbertTree;

    use super::*;

    #[test]
    fn test_basic() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let tree = HilbertTree::open(&dir, 12).unwrap();
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_ext = populate_hilbert_leaves_external(
            &dir,
            &tree.archive,
            &m_node_pairs,
            &m_way_pairs,
            &tree.leaves,
            tree.leaf_zoom,
        )
        .unwrap();
        let ext = m_ext.slice();
        assert_eq!(ext.len(), 186);

        let mut osm_id = 0;
        let mut count = 0;
        let ways = tree.archive.ways();
        for i in 0..ext.len() {
            let w = &ways[i];
            osm_id = w.osm_id();
            count += 1;
        }
        assert_eq!(count, ext.len());
        assert_eq!(osm_id, 219347304);
    }
}
