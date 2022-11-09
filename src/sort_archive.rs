use crate::{
    location,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{
        HilbertNodePair, HilbertWayPair, Node, NodeIndex, Osm, TagIndex, Way,
    },
};
use geo::algorithm::interior_point::InteriorPoint;
use geo::geometry::{Coordinate, LineString};
use geo::{coord, Polygon};
use log::info;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::{
    io::{Error, ErrorKind, Stdout},
    path::PathBuf,
    time::Instant,
};

pub fn sort(archive: Osm, dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match archive.hilbert_node_pairs() {
        Some(p) => p,
        None => {
            return Err(Box::new(Error::new(
                ErrorKind::NotFound,
                "No hilbert node pairs!",
            )));
        }
    };

    // Build hilbert way pairs.
    let ways = archive.ways();
    let ways_len = archive.ways().len();
    let way_pairs_mut = Mutant::<HilbertWayPair>::new(dir, "hilbert_way_pairs", ways_len)?;
    let way_pairs = way_pairs_mut.mutable_slice();
    build_hilbert_way_pairs(way_pairs, &archive)?;

    // Sort hilbert way pairs.
    info!("Sorting hilbert way pairs.");
    let t = Instant::now();
    way_pairs.par_sort_unstable_by_key(|idx| idx.h());
    info!("Finished in {} secs.", t.elapsed().as_secs());

    // Sort hilbert node pairs.
    info!("Sorting hilbert node pairs.");
    let t = Instant::now();
    let nodes_len = archive.nodes().len();
    let node_pairs_mut = Mutant::<HilbertNodePair>::new(dir, "hilbert_node_pairs", nodes_len)?;
    let node_pairs = node_pairs_mut.mutable_slice();
    node_pairs.par_sort_unstable_by_key(|idx| idx.h());
    info!("Finished in {} secs.", t.elapsed().as_secs());

    // Reorder nodes to sorted hilbert node pairs.
    let mut pb = Prog::new("Reordering nodes. ", nodes_len);
    let nodes = archive.nodes();
    let sorted_nodes_mut = Mutant::<Node>::new(&dir, "sorted_nodes", nodes_len)?;
    let sorted_nodes = sorted_nodes_mut.mutable_slice();
    let mut tag_counter: usize = 0;
    let tags_index = archive.tags_index();
    let tags_index_len = tags_index.len();
    let sorted_tags_index_mut = Mutant::<TagIndex>::new(dir, "sorted_tags_index", tags_index_len)?;
    let sorted_tags_index = sorted_tags_index_mut.mutable_slice();
    sorted_nodes.iter_mut().zip(node_pairs.iter_mut()).for_each(
        |(sorted_node, hilbert_node_pair)| {
            let i = hilbert_node_pair.i() as usize;
            let node = &nodes[i];
            let start = node.tag_first_idx() as usize;
            let end = node.tags().end as usize;

            let tag_first_idx = tag_counter;
            for t in &tags_index[start..end] {
                sorted_tags_index[tag_counter].fill_from(t);
                tag_counter += 1;
            }

            sorted_node.fill_from(node);
            sorted_node.set_tag_first_idx(tag_first_idx as u64);
            pb.tick(i);
        },
    );
    pb.finish();

    // Reorder ways to sorted hilbert way pairs.
    let mut pb = Prog::new("Reordering ways. ", ways_len);
    let sorted_ways_mut = Mutant::<Way>::new(dir, "sorted_ways", ways_len)?;
    let sorted_ways = sorted_ways_mut.mutable_slice();
    let mut nodes_index_counter: usize = 0;
    let nodes_index = archive.nodes_index();
    let nodes_index_len = nodes_index.len();
    let sorted_nodes_index_mut =
        Mutant::<NodeIndex>::new(dir, "sorted_nodes_index", nodes_index_len)?;
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

    Ok(())
}

fn build_hilbert_way_pairs(
    way_pairs: &mut [HilbertWayPair],
    archive: &Osm,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes = archive.nodes();
    let nodes_index = archive.nodes_index();
    let ways = archive.ways();

    info!("Building hilbert way pairs.");
    let t = Instant::now();

    way_pairs.par_iter_mut().enumerate().for_each(|(i, pair)| {
        let way = &ways[i];
        let refs = way.refs();
        let len = refs.end - refs.start;
        let mut coords = Vec::<Coordinate<f64>>::with_capacity(len as usize);

        for r in refs {
            if let Some(idx) = nodes_index[r as usize].value() {
                let node = &nodes[idx as usize];
                // georust lib requires f64 for a coordinate.
                coords.push(coord! { x: node.lon() as f64, y: node.lat() as f64 });
            };
        }

        // Calculate point on surface.
        // http://libgeos.org/doxygen/classgeos_1_1algorithm_1_1InteriorPointArea.html
        // https://docs.rs/geo/latest/geo/algorithm/interior_point/trait.InteriorPoint.html
        // https://github.com/georust/geo/blob/main/geo/src/algorithm/interior_point.rs

        let point_on_surface = if coords.first() == coords.last() {
            Polygon::new(LineString::new(coords), vec![]).interior_point()
        } else {
            LineString::new(coords).interior_point()
        };

        if let Some(pos) = point_on_surface {
            let lonlat = (pos.x() as i32, pos.y() as i32);
            // info!("way point on surface {:#?}", lonlat);
            let h = location::lonlat_to_h(lonlat);

            pair.set_i(i as u64);
            pair.set_h(h);
        } else {
            eprintln!(
                "Unable to find point on surface to compute hilbert location for way at index {}.",
                i
            );
        }
    });

    info!("Finished in {} secs.", t.elapsed().as_secs());
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
        info!("Finished in {} secs.", self.t.elapsed().as_secs());
    }
}

#[cfg(test)]
mod tests {
    use crate::location;

    use super::*;

    #[test]
    fn test_a_few_hilbert_pairs() {
        let d = PathBuf::from("/Users/n/geodata/flatdata/santacruz");
        let m = Mutant::<HilbertNodePair>::open(&d, "hilbert_node_pairs", true).unwrap();
        let ns = m.slice();
        let m2 = Mutant::<HilbertWayPair>::open(&d, "hilbert_way_pairs", true).unwrap();
        let ws = m2.slice();

        // Print a sanity check
        let mut it = ws.iter();
        for n in &ns[0..5] {
            let w = it.next().unwrap();
            let nh = n.h();
            let wh = w.h();
            let n_lonlat = location::h_to_decimal_lonlat(nh);
            let w_lonlat = location::h_to_decimal_lonlat(wh);

            println!("n {} w {}", n.h(), w.h());
            println!("n {:?} w {:?}", n_lonlat, w_lonlat);
        }

        let n = ns.first().unwrap();
        let w = ws.first().unwrap();
        let n_h = n.h();
        let w_h = w.h();
        let n_coord = fast_hilbert::h2xy::<u32>(n_h, 32);
        let w_coord = fast_hilbert::h2xy::<u32>(w_h, 32);
        let (n_lon, n_lat) = crate::location::xy_to_decimal_lonlat(n_coord);
        let (w_lon, w_lat) = crate::location::xy_to_decimal_lonlat(w_coord);

        // println!("n {} w {}", n_h, w_h);
        // println!("n {:?} {:?} w {:?} {:?}", n_lon, n_lat, w_lon, w_lat);

        // assert_eq!(n_h, 5055384463774690405);
        // assert_eq!(w_h, 5056311634151337179);
        // assert_eq!(n_lon, -122.4891643);
        // assert_eq!(n_lat, 36.9479216);
        // assert_eq!(w_lon, -122.1471753);
        // assert_eq!(w_lat, 37.2459612);
    }
}
