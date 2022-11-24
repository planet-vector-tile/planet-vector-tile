use std::ops::Range;

use ahash::AHashMap;
use flatdata::RawData;

use crate::{
    manifest::Manifest,
    osmflat::osmflat_generated::osm::{Node, Osm, Tag, TagIndex},
};

pub struct Filter<'a> {
    archive: &'a Osm,
    str_to_idx: AHashMap<&'a str, usize>,
}

impl<'a> Filter<'a> {
    pub fn new(manifest: &'a Manifest, archive: &'a Osm) -> Filter<'a> {
        // The str of a string in the rules within the manifest
        // mapped to the index of the substring within the strings
        // vector of the archive. This caching reduces the thrash of
        // lookups in the giant strings vector.
        let mut str_to_idx: AHashMap<&'a str, usize> = AHashMap::new();

        // The order of precedence for the evaluation of rules is to look 
        // at tags first, then values, then keys.
        let mut tag_to_zoom_range: AHashMap<(&'a str, &'a str), Range<u8>> = AHashMap::new();
        let mut value_to_zoom_range: AHashMap<&'a str, Range<u8>> = AHashMap::new();
        let mut key_to_zoom_range: AHashMap<&'a str, Range<u8>> = AHashMap::new();

        for (_, rule) in &manifest.rules {
            let zoom_range = match rule.maxzoom {
                Some(maxzoom) => rule.minzoom..maxzoom,
                None => rule.minzoom..manifest.render.leaf_zoom,
            };
            for tag in &rule.tags {
                tag_to_zoom_range.insert((&tag.0, &tag.1), zoom_range.clone());
            }
            for value in &rule.values {
                value_to_zoom_range.insert(value, zoom_range.clone());
            }
            for key in &rule.keys {
                key_to_zoom_range.insert(key, zoom_range.clone());
            }
        }

        Filter {
            archive,
            str_to_idx,
        }
    }

    // https://stackoverflow.com/questions/25445761/returning-a-closure-from-a-function
    pub fn node_filter(&self, zoom: u8) -> impl Fn(&Node) -> bool {
        let evaluate_node = |node: &Node| -> bool {
            // node.tags()

            true
        };

        evaluate_node
    }

    // https://stackoverflow.com/questions/41269043/how-would-one-return-a-function-from-a-function-in-rust
    pub fn node_filter2(&self, zoom: u8) -> fn(&Node) -> bool {
        fn evaluate_node(node: &Node) -> bool {
            let mut result = true;

            result
        }

        evaluate_node
    }
}

fn evaluate_tags(
    tags_index_range: Range<usize>,
    tags_index: &[TagIndex],
    tags: &[Tag],
    strings: RawData,
) -> bool {
    let mut result = true;

    result
}
