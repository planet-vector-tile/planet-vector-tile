use crate::osmflat::osmflat_generated::osm::EntityType;
use crate::{
    location,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{
        HilbertNodePair, HilbertRelationPair, HilbertWayPair, Node, NodeIndex, Osm, Relation,
        TagIndex, Way,
    },
    util::{self, finish, timer},
};
use crossbeam::queue::SegQueue;
use geo::algorithm::interior_point::InteriorPoint;
use geo::geometry::LineString;
use geo::Coord;
use geo::{coord, Polygon};
use pbr::ProgressBar;
use rayon::prelude::*;
use std::{
    fs,
    io::{Error, ErrorKind, Stdout},
    panic,
    path::PathBuf,
    time::Instant,
};

pub fn sort_flatdata(flatdata: Osm, dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match flatdata.hilbert_node_pairs() {
        Some(p) => p,
        None => {
            return Err(Box::new(Error::new(
                ErrorKind::NotFound,
                "No hilbert node pairs!",
            )));
        }
    };

    // Build hilbert way pairs.
    let ways = flatdata.ways();
    let ways_len = flatdata.ways().len();
    let m_way_pairs = Mutant::<HilbertWayPair>::new(dir, "hilbert_way_pairs", ways_len)?;
    let way_pairs = m_way_pairs.mutable_slice();
    build_hilbert_way_pairs(way_pairs, &flatdata)?;

    // Sort hilbert way pairs.
    let t = util::timer("Sorting hilbert way pairs.");
    way_pairs.par_sort_unstable_by_key(|idx| idx.h());
    println!("Finished in {} secs.", t.elapsed().as_secs());

    // Build hilbert relation pairs
    let m_relation_pairs = Mutant::<HilbertRelationPair>::new(
        dir,
        "hilbert_relation_pairs",
        flatdata.relations().len(),
    )?;
    build_hilbert_relation_pairs(&m_way_pairs, &m_relation_pairs, &flatdata)?;

    // Sort hilbert node pairs.
    let t = util::timer("Sorting hilbert node pairs.");
    let nodes_len = flatdata.nodes().len();
    let node_pairs_mut = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
    let node_pairs = node_pairs_mut.mutable_slice();
    node_pairs.par_sort_unstable_by_key(|idx| idx.h());
    println!("Finished in {} secs.", t.elapsed().as_secs());

    // Reorder nodes to sorted hilbert node pairs.
    let mut pb = Prog::new("Reordering nodes. ", nodes_len);
    let nodes = flatdata.nodes();
    let mut m_sorted_nodes = Mutant::<Node>::new_from_flatdata(&dir, "sorted_nodes", "nodes")?;
    let sorted_nodes = m_sorted_nodes.mutable_slice();
    let m_old_node_idx = Mutant::<usize>::new(dir, "old_node_idx", nodes_len)?;
    let old_node_idx = m_old_node_idx.mutable_slice();
    let mut tag_counter: usize = 0;
    let tags_index = flatdata.tags_index();
    let mut sorted_tags_index_mut =
        Mutant::<TagIndex>::new_from_flatdata(dir, "sorted_tags_index", "tags_index")?;
    let sorted_tags_index = sorted_tags_index_mut.mutable_slice();
    for i in 0..nodes_len {
        let node_pair = &node_pairs[i];
        let old_i = node_pair.i() as usize;
        old_node_idx[old_i] = i;
        let node = &nodes[old_i];
        let sorted_node = &mut sorted_nodes[i];

        let tags_range = node.tags();
        let start = tags_range.start as usize;
        let end = tags_range.end as usize;

        let tag_first_idx = tag_counter;
        for t in &tags_index[start..end] {
            sorted_tags_index[tag_counter].fill_from(t);
            tag_counter += 1;
        }

        sorted_node.fill_from(&node);
        sorted_node.set_tag_first_idx(tag_first_idx as u64);
        pb.tick(i);
    }
    pb.finish();

    // Update references in nodes_index.
    let mut pb = Prog::new("Updating node references in nodes_index. ", nodes_len);
    let m_nodes_index = Mutant::<NodeIndex>::open(dir, "nodes_index", true)?;
    let nodes_index = m_nodes_index.mutable_slice();
    let nodes_index_len = nodes_index.len();
    for i in 0..nodes_index_len {
        let node_index = &mut nodes_index[i];
        let old_i = node_index.value().unwrap() as usize;
        let new_i = old_node_idx[old_i];
        node_index.set_value(Some(new_i as u64));
        pb.tick(i);
    }

    // Remove the old node index, as we don't need it anymore.
    let old_node_idx_path = m_old_node_idx.path.clone();
    // drop(m_old_node_idx);
    let _ = fs::remove_file(old_node_idx_path);

    // Reorder ways to sorted hilbert way pairs.
    let mut pb = Prog::new("Reordering ways. ", ways_len);
    let mut sorted_ways_mut = Mutant::<Way>::new_from_flatdata(dir, "sorted_ways", "ways")?;
    let sorted_ways = sorted_ways_mut.mutable_slice();
    let mut nodes_index_counter: usize = 0;
    let mut sorted_nodes_index_mut =
        Mutant::<NodeIndex>::new_from_flatdata(dir, "sorted_nodes_index", "nodes_index")?;
    let sorted_nodes_index = sorted_nodes_index_mut.mutable_slice();
    sorted_ways
        .iter_mut()
        .zip(way_pairs.iter_mut())
        .for_each(|(sorted_way, hilbert_way_pair)| {
            let i = hilbert_way_pair.i() as usize;
            let way = &ways[i];

            let start = way.tag_first_idx() as usize;
            let end = way.tags().end as usize;

            let tag_first_idx = tag_counter;
            for t in &tags_index[start..end] {
                sorted_tags_index[tag_counter].fill_from(t);
                tag_counter += 1;
            }

            let ref_start = way.ref_first_idx() as usize;
            let ref_end = way.refs().end as usize;

            let nodes_first_idx = nodes_index_counter;
            for r in &nodes_index[ref_start..ref_end] {
                sorted_nodes_index[nodes_index_counter].fill_from(r);
                nodes_index_counter += 1;
            }

            sorted_way.fill_from(way);
            sorted_way.set_tag_first_idx(tag_first_idx as u64);
            sorted_way.set_ref_first_idx(nodes_first_idx as u64);
            pb.tick(i);
        });
    pb.finish();

    // std::mem::drop(flatdata);
    m_sorted_nodes.mv("nodes")?;
    println!("Moved sorted_nodes to nodes");
    sorted_ways_mut.mv("ways")?;
    println!("Moved sorted_ways to ways");
    sorted_nodes_index_mut.mv("nodes_index")?;
    println!("Moved sorted_nodes_index to nodes_index");
    sorted_tags_index_mut.mv("tags_index")?;
    println!("Moved sorted_tags_index to tags_index");

    Ok(())
}

fn build_hilbert_way_pairs(
    way_pairs: &mut [HilbertWayPair],
    flatdata: &Osm,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes = flatdata.nodes();
    let nodes_index = flatdata.nodes_index();
    let node_pairs = flatdata.hilbert_node_pairs().unwrap();
    let ways = flatdata.ways();

    let t = timer("Building hilbert way pairs.");

    way_pairs.par_iter_mut().enumerate().for_each(|(i, pair)| {
        let way = &ways[i];
        let refs = way.refs();
        let len = refs.end - refs.start;
        let refs_start = refs.clone().start as usize;

        // invalid ring
        // https://github.com/georust/geo/blob/3b0d5738f54bd8964f7d1f573bd63dc114587dc4/geo/src/algorithm/relate/geomgraph/geometry_graph.rs#L182
        if len < 4 {
            if let Some(idx) = nodes_index[refs_start].value() {
                let h = node_pairs[idx as usize].h();
                pair.set_i(i as u32);
                pair.set_h(h);
            } else {
                println!(
                    "Unable to find a hilbert location for way with {} refs. i={} osm_id={}",
                    len,
                    i,
                    way.osm_id()
                );
            }
            return;
        }

        let mut coords = Vec::<Coord<f64>>::with_capacity(len as usize);

        for r in refs {
            if let Some(idx) = nodes_index[r as usize].value() {
                let node = &nodes[idx as usize];
                // georust lib requires f64 for a coordinate.
                coords.push(coord! { x: node.lon() as f64, y: node.lat() as f64 });
            };
        }

        if coords.len() < 4 {
            if let Some(idx) = nodes_index[refs_start].value() {
                let h = node_pairs[idx as usize].h();
                pair.set_i(i as u32);
                pair.set_h(h);
            } else {
                println!(
                    "way refs >= 4 but coords.len() < 4 for way with {} refs. i={} osm_id={}",
                    len,
                    i,
                    way.osm_id()
                );
            }
            return;
        }

        // Calculate point on surface.
        // http://libgeos.org/doxygen/classgeos_1_1algorithm_1_1InteriorPointArea.html
        // https://docs.rs/geo/latest/geo/algorithm/interior_point/trait.InteriorPoint.html
        // https://github.com/georust/geo/blob/main/geo/src/algorithm/interior_point.rs

        // NHTODO https://crates.io/crates/polylabel

        let point_on_surface_res = panic::catch_unwind(|| {
            let point_on_surface = if coords.first() == coords.last() {
                Polygon::new(LineString::new(coords), vec![]).interior_point()
            } else {
                LineString::new(coords).interior_point()
            };
            point_on_surface
        });

        let point_on_surface = match point_on_surface_res {
            Ok(point_on_surface) => point_on_surface,
            Err(e) => {
                eprintln!(
                    "interior_point panic for way with {} refs. i={} osm_id={} Error: {:?}",
                    len,
                    i,
                    way.osm_id(),
                    e
                );
                None
            }
        };

        if let Some(pos) = point_on_surface {
            let lonlat = (pos.x() as i32, pos.y() as i32);
            let h = location::lonlat_to_h(lonlat);
            pair.set_i(i as u32);
            pair.set_h(h);
        } else {
            println!(
                "Unable to find point on surface for way at index {}. osm_id={}.",
                i,
                way.osm_id()
            );
            let refs = way.refs();
            let median_ref = refs.start + ((refs.end - refs.start) / 2);
            if let Some(idx) = nodes_index[median_ref as usize].value() {
                let h = node_pairs[idx as usize].h();
                pair.set_i(i as u32);
                pair.set_h(h);
                println!(
                    "Using median ref of way for hilbert location at index {}. osm_id={}.",
                    i,
                    way.osm_id()
                );
            }
            // Also trying the first ref if the median doesn't work.
            else if let Some(idx) = nodes_index[refs.start as usize].value() {
                let h = node_pairs[idx as usize].h();
                pair.set_i(i as u32);
                pair.set_h(h);
                println!(
                    "Using first ref of way for hilbert location at index {}. osm_id={}.",
                    i,
                    way.osm_id()
                );
            } else {
                println!(
                    "Unable to find hilbert location for way at index {}. osm_id={}.",
                    i,
                    way.osm_id()
                );
            }
        }
    });

    finish(t);
    Ok(())
}

fn build_hilbert_relation_pairs(
    m_way_pairs: &Mutant<HilbertWayPair>,
    m_relation_pairs: &Mutant<HilbertRelationPair>,
    flatdata: &Osm,
) -> Result<(), Box<dyn std::error::Error>> {
    let t = timer("Building hilbert relation pairs.");

    // For relations with relation members, we need to dig into the members to find
    // their Hilbert locations, so we need to have a queue of relations waiting for that.
    let q = SegQueue::<usize>::new();

    let node_pairs = flatdata.hilbert_node_pairs().unwrap();
    let way_pairs = m_way_pairs.slice();
    let relation_pairs = m_relation_pairs.slice();

    let relations = flatdata.relations();
    let members = flatdata.members();

    let compute_relation_h = |relation_i: usize, relation: &Relation| {
        let relation_pair = &mut m_relation_pairs.mutable_slice()[relation_i];
        relation_pair.set_i(relation_i as u32);

        let mut h_total: u128 = 0;
        let mut processed_members_count: u32 = 0;

        let members_range = relation.members();

        let mut missing_member = false;
        for member in &members[members_range.start as usize..members_range.end as usize] {
            let idx = member.idx();
            if idx.is_none() {
                missing_member = true;
                continue;
            }
            let i = member.idx().unwrap() as usize;
            let h = match member.entity_type() {
                EntityType::Node => node_pairs[i].h(),
                EntityType::Way => way_pairs[i].h(),
                EntityType::Relation => {
                    let h = relation_pairs[i].h();
                    if h == 0 {
                        // Here we add the relation to the queue to be processed later.
                        q.push(i as usize);
                        return;
                    }
                    h
                }
                _ => 0,
            };

            h_total += h as u128;
            processed_members_count += 1;
        }
        if missing_member {
            println!(
                "Missing member(s) for relation. osm_id={}",
                relation.osm_id()
            );
        }
        let mean_h = (h_total / processed_members_count as u128) as u64;
        relation_pair.set_h(mean_h);
    };

    relations
        .par_iter()
        .enumerate()
        .for_each(|(relation_i, relation)| compute_relation_h(relation_i, relation));

    let mut last_q_len = q.len();
    let mut try_count: u8 = 0;
    while let Some(relation_i) = q.pop() {
        compute_relation_h(relation_i, &relations[relation_i]);
        if q.len() == last_q_len {
            try_count += 1;
        }
        if try_count == 100 {
            // We should continue along with the build, so we should just log the problema and move on.
            eprintln!(
                "Unable to compute all of the h for relations. Re-tried {} times.",
                try_count
            );
        }
        last_q_len = q.len();
    }

    finish(t);
    Ok(())
}

struct Prog {
    prog_counter: usize,
    pb: ProgressBar<Stdout>,
    t: Instant,
}

impl Prog {
    fn new(msg: &str, len: usize) -> Self {
        let prog_len = len as u64 / 1_000_000;
        let t = Instant::now();
        let mut pb = ProgressBar::new(prog_len);
        pb.message(msg);
        Self {
            prog_counter: 0,
            pb,
            t,
        }
    }
    fn tick(&mut self, i: usize) {
        if i / 1_000_000 > self.prog_counter {
            self.pb.inc();
            self.prog_counter += 1;
        }
    }
    fn finish(&self) {
        println!("Finished in {} secs.", self.t.elapsed().as_secs());
    }
}

#[cfg(test)]
mod tests {
    use flatdata::FileResourceStorage;

    use super::*;

    #[test]
    fn test_a_few_hilbert_pairs() {
        let d = PathBuf::from("tests/fixtures/santa_cruz/sort");
        let m = Mutant::<HilbertNodePair>::open(&d, "hilbert_node_pairs", true).unwrap();
        let ns = m.slice();
        let m2 = Mutant::<HilbertWayPair>::open(&d, "hilbert_way_pairs", true).unwrap();
        let ws = m2.slice();

        let n = ns.first().unwrap();
        let w = ws.first().unwrap();
        let n_h = n.h();
        let w_h = w.h();
        let n_coord = fast_hilbert::h2xy::<u32>(n_h, 32);
        let w_coord = fast_hilbert::h2xy::<u32>(w_h, 32);
        let (n_lon, n_lat) = crate::location::xy_to_decimal_lonlat(n_coord);
        let (w_lon, w_lat) = crate::location::xy_to_decimal_lonlat(w_coord);

        assert_eq!(n_h, 3660331851833214363);
        assert_eq!(w_h, 3660337306988711752);
        assert_eq!(n_lon, -121.2510385);
        assert_eq!(n_lat, 36.9596099);
        assert_eq!(w_lon, -121.4516216);
        assert_eq!(w_lat, 36.9000422);
    }

    #[test]
    fn test_tags_index() {
        let dir = PathBuf::from("tests/fixtures/santa_cruz/sort");
        let m_nodes = Mutant::<Node>::open(&dir, "nodes", true).unwrap();
        let nodes = m_nodes.slice();
        for n in nodes {
            let range = n.tags();
            assert!(range.start <= range.end || range.end == 0);
        }
    }

    #[test]
    fn test_way_ref_under_nodes_idx_len() {
        let dir = PathBuf::from("tests/fixtures/santa_cruz/sort");
        let m_ways = Mutant::<Way>::open(&dir, "ways", true).unwrap();
        let ways = m_ways.slice();
        let w = &ways[186316];
        let osm_id = w.osm_id();
        assert_eq!(osm_id, 42298798);

        let first_ref = w.ref_first_idx() as usize;

        let m_nodes_idx = Mutant::<NodeIndex>::open(&dir, "nodes_index", true).unwrap();
        let n_idx = m_nodes_idx.slice();
        let n_idx_len = n_idx.len();

        assert_eq!(m_nodes_idx.len, n_idx_len);
        assert!((first_ref as usize) < n_idx.len());
    }

    #[test]
    #[ignore]
    fn test_build_hilbert_way_pairs_planet() {
        let dir = PathBuf::from("/Users/n/geodata/flatdata/planet");
        let flatdata = Osm::open(FileResourceStorage::new(&dir)).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();
        let way_pairs = m_way_pairs.mutable_slice();
        let _ = build_hilbert_way_pairs(way_pairs, &flatdata);
    }

    #[test]
    fn test_hilbert_relation_pairs() {
        let dir = PathBuf::from("tests/fixtures/santa_cruz/sort");
        let relation_pair_path =
            PathBuf::from("tests/fixtures/santa_cruz/sort/hilbert_relation_pairs");
        let _ = fs::remove_file(relation_pair_path);
        let flatdata = Osm::open(FileResourceStorage::new(&dir)).unwrap();

        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", false).unwrap();

        let len = flatdata.relations().len();
        let m_relation_pairs =
            Mutant::<HilbertRelationPair>::new(&dir, "hilbert_relation_pairs", len).unwrap();

        build_hilbert_relation_pairs(&m_way_pairs, &m_relation_pairs, &flatdata).unwrap();
        println!("after");
        for p in m_relation_pairs.slice() {
            println!("h {} i {}", p.h(), p.i());
        }

        // just a basic sanity check to see if we don't panic
        assert!(m_relation_pairs.slice()[3243].h() > 1);
    }
}
