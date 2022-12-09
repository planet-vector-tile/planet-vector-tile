use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::manifest::Manifest;
use crate::osmflat::osmflat_generated::osm::Osm;
use flatdata::FileResourceStorage;
use itertools::Itertools;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub fn survey(manifest: Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let planet = &manifest.data.planet;

    let flatdata = Osm::open(FileResourceStorage::new(planet))?;

    let tags_idx = flatdata.tags_index();
    let tags_idx_len = tags_idx.len();
    let tags = flatdata.tags();
    let strings = flatdata.stringtable();

    let nodes = flatdata.nodes();
    let ways = flatdata.ways();
    let relations = flatdata.relations();

    let total_nodes = nodes.len();
    let total_ways = ways.len();
    let total_relations = relations.len();

    let mut nodes_count = AtomicU64::new(0);

    flatdata.nodes().par_iter().for_each(|n| {
        let osm_id = n.osm_id();
        let ts = n.tags();
        if ts.start == ts.end {
            return;
        }
        let tag_range = ts.start as usize..(if ts.end == 0 {
            tags_idx_len
        } else {
            ts.end as usize
        });

        let mut tag_dict: BTreeMap<&str, &str> = BTreeMap::new();

        for t_i in &tags_idx[tag_range] {
            let t = &tags[t_i.value() as usize];
            let k = unsafe { strings.substring_unchecked(t.key_idx() as usize) };
            let v = unsafe { strings.substring_unchecked(t.value_idx() as usize) };
            tag_dict.insert(k, v);
        }

        let tags_str = tag_dict
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .join(" ");

        if tag_dict.contains_key("place") {
            nodes_count.fetch_add(1, Ordering::Relaxed);
            println!("n {} {}", osm_id, tags_str);
        }
    });

    let n_count = *nodes_count.get_mut();
    println!("node matches: {} / {} - {:.5} %", n_count, total_nodes, n_count as usize / total_nodes * 100);

    flatdata.ways().par_iter().for_each(|w| {});

    flatdata.relations().par_iter().for_each(|r| {});

    Ok(())
}
