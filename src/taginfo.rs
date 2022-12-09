use std::{
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::manifest::Manifest;
use crate::osmflat::osmflat_generated::osm::{Osm, Tag, TagIndex};
use flatdata::{FileResourceStorage, RawData};
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
        if n.osm_id() == 316949491 {
            println!("should be pulaski")
        }
        let Some(tags) = tags_vec(n.tags(), tags_idx, tags, strings) else { return };
        let Some(ts ) = place(tags) else { return };

        println!("n {} {}", n.osm_id(), tags_string(ts));
        nodes_count.fetch_add(1, Ordering::Relaxed);
    });

    let n_count = *nodes_count.get_mut();
    println!(
        "node matches: {} / {} - {:.5} %",
        n_count,
        total_nodes,
        n_count as usize / total_nodes * 100
    );

    flatdata.ways().par_iter().for_each(|w| {});

    flatdata.relations().par_iter().for_each(|r| {});

    Ok(())
}

pub fn tags_vec<'a>(
    tags_idx_range: Range<u64>,
    tags_idx: &'a [TagIndex],
    tags: &'a [Tag],
    strings: RawData<'a>,
) -> Option<Vec<(&'a str, &'a str)>> {
    if tags_idx_range.start == tags_idx_range.end {
        return None;
    }

    let tag_range = tags_idx_range.start as usize..(if tags_idx_range.end == 0 {
        tags_idx.len()
    } else {
        tags_idx_range.end as usize
    });

    let mut vec: Vec<(&'a str, &'a str)> = Vec::with_capacity(tag_range.end - tag_range.start);
    for i in &tags_idx[tag_range] {
        let t = &tags[i.value() as usize];
        let k = unsafe { strings.substring_unchecked(t.key_idx() as usize) };
        let v = unsafe { strings.substring_unchecked(t.value_idx() as usize) };
        vec.push((k, v));
    }
    Some(vec)
}

fn sort<'a>(tags: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    let mut sorted = tags.clone();
    sorted.sort_unstable_by_key(|(k, _)| *k);
    sorted
}

fn tags_string<'a>(tags: Vec<(&'a str, &'a str)>) -> String {
    tags.iter().map(|(k, v)| format!("{}={}", k, v)).join(" ")
}

pub fn place<'a>(tags: Vec<(&'a str, &'a str)>) -> Option<Vec<(&'a str, &'a str)>> {
    let m = tags.iter().find(|(k, v)| *k == "place");
    if m.is_some() {
        Some(sort(tags))
    } else {
        None
    }
}
