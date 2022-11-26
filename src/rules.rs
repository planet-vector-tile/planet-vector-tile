use std::{ops::Range, time::Instant};

use ahash::AHashMap;
use dashmap::{DashMap, DashSet};
use flatdata::RawData;
use humantime::format_duration;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    manifest::Manifest,
    osmflat::osmflat_generated::osm::{Osm, Tag},
};

pub struct Rules {
    tag_to_zoom_range: AHashMap<(usize, usize), Range<u8>>,
    value_to_zoom_range: AHashMap<usize, Range<u8>>,
    key_to_zoom_range: AHashMap<usize, Range<u8>>,
}

impl Rules {
    // NOTE: This is expensive to construct due to get_strs. Don't construct in a loop.
    pub fn new(manifest: &Manifest, archive: &Osm) -> Self {
        let rule_strs: DashSet<&str> = DashSet::new();
        for (_, rule) in &manifest.rules {
            for (k, v) in &rule.tags {
                rule_strs.insert(k);
                rule_strs.insert(v);
            }
            for v in &rule.values {
                rule_strs.insert(v);
            }
            for k in &rule.keys {
                rule_strs.insert(k);
            }
        }

        let str_to_idx: DashMap<&str, usize> = DashMap::new();
        let strings = archive.stringtable();
        let t = Instant::now();

        // Note: This is expensive.
        let strs = get_strs(strings);

        let _ = strs.par_iter().enumerate().find_any(|(i, &s)| {
            if rule_strs.contains(s) {
                str_to_idx.insert(s, *i);
                rule_strs.remove(s);
            }
            // halt iterating when the set is empty
            if rule_strs.is_empty() {
                true
            } else {
                false
            }
        });

        if rule_strs.len() > 0 {
            println!("WARNING: Not all rules were matched to a string in the stringtable. Unmatched strings : {:?}", rule_strs);
        }
        println!("Rules str_to_index: {:?}", str_to_idx);
        println!("Rules str_to_index time: {}", format_duration(t.elapsed()));

        let mut tag_to_zoom_range: AHashMap<(usize, usize), Range<u8>> = AHashMap::new();
        let mut value_to_zoom_range: AHashMap<usize, Range<u8>> = AHashMap::new();
        let mut key_to_zoom_range: AHashMap<usize, Range<u8>> = AHashMap::new();

        for (_, rule) in &manifest.rules {
            let zoom_range = if let Some(maxzoom) = rule.maxzoom {
                rule.minzoom..maxzoom
            } else {
                rule.minzoom..manifest.render.leaf_zoom
            };

            for (k, v) in &rule.tags {
                let k_idx = match str_to_idx.get(k.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                let v_idx = match str_to_idx.get(v.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                tag_to_zoom_range.insert((k_idx, v_idx), zoom_range.clone());
            }
            for v in &rule.values {
                let v_idx = match str_to_idx.get(v.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                value_to_zoom_range.insert(v_idx, zoom_range.clone());
            }
            for k in &rule.keys {
                let k_idx = match str_to_idx.get(k.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                key_to_zoom_range.insert(k_idx, zoom_range.clone());
            }
        }

        Rules {
            tag_to_zoom_range,
            value_to_zoom_range,
            key_to_zoom_range,
        }
    }

    pub fn get_zoom_range(&self, tag: &Tag) -> ZoomRangeRuleEval {
        let key = tag.key_idx() as usize;
        let value = tag.value_idx() as usize;
        if let Some(zoom_range) = self.tag_to_zoom_range.get(&(key, value)) {
            return ZoomRangeRuleEval::Tag(zoom_range);
        }
        if let Some(zoom_range) = self.value_to_zoom_range.get(&value) {
            return ZoomRangeRuleEval::Value(zoom_range);
        }
        if let Some(zoom_range) = self.key_to_zoom_range.get(&key) {
            return ZoomRangeRuleEval::Key(zoom_range);
        }
        ZoomRangeRuleEval::None
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ZoomRangeRuleEval<'a> {
    None,
    Tag(&'a Range<u8>),
    Value(&'a Range<u8>),
    Key(&'a Range<u8>),
}

fn get_str_null_delimeters(strings: RawData) -> Vec<usize> {
    let bytes = strings.as_bytes();
    // Serial - 2s 135ms 674us 708ns for california
    // let mut delimeters: Vec<usize> = Vec::new();
    // for (i, byte) in bytes.iter().enumerate() {
    //     if *byte == 0 {
    //         delimeters.push(i);
    //     }
    // }

    // Parallel - 570ms 808us 375ns for california
    let delimeters: Vec<usize> = bytes
        .par_iter()
        .enumerate()
        .filter_map(|(i, byte)| if *byte == 0 { Some(i) } else { None })
        .collect();

    delimeters
}

fn get_strs(strings: RawData) -> Vec<&str> {
    let bytes = strings.as_bytes();
    let delimeters = get_str_null_delimeters(strings);

    // Serial - 1s 77ms 806us 708ns for california
    // let mut strs = Vec::with_capacity(delimeters.len());
    // let str0 = unsafe { std::str::from_utf8_unchecked(&bytes[..delimeters[0]]) };
    // strs.push(str0);
    // for i in 1..delimeters.len() {
    //     let start = delimeters[i - 1] + 1;
    //     let end = delimeters[i];
    //     let str = unsafe { std::str::from_utf8_unchecked(&bytes[start..end]) };
    //     strs.push(str);
    // }

    // Parallel - 570ms 808us 375ns for california
    let strs: Vec<&str> = delimeters
        .par_iter()
        .enumerate()
        .map(|(i, delimeter)| {
            let start = if i == 0 { 0 } else { delimeters[i - 1] + 1 };
            let end = *delimeter;
            let slice = &bytes[start..end];
            let str = unsafe { std::str::from_utf8_unchecked(slice) };
            str
        })
        .collect();

    strs
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use flatdata::FileResourceStorage;
    use humantime::format_duration;

    use crate::manifest;

    use super::*;

    #[test]
    fn test_get_str_indices() {
        let strings = RawData::new(b"hello\0world\0this\0is\0a\0test\0");
        let indices = get_str_null_delimeters(strings);
        assert_eq!(indices, vec![5, 11, 16, 19, 21, 26]);
    }

    #[test]
    fn test_get_strs() {
        let strings = RawData::new(b"hello\0world\0this\0is\0a\0test\0");
        let strs = get_strs(strings);
        assert_eq!(strs, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_get_strs_santacruz() {
        let archive = Osm::open(FileResourceStorage::new("tests/fixtures/santacruz/sort")).unwrap();
        let strings: RawData = archive.stringtable();
        let delimeters = get_str_null_delimeters(strings);
        let d1 = delimeters[0];
        assert_eq!(d1, 8);
        assert_eq!(delimeters.len(), 60755);

        let strs = get_strs(strings);
        assert_eq!(strs.len(), delimeters.len());

        assert_eq!(
            &strs[..20],
            vec![
                "osmflatc",
                "",
                "Speight",
                "stevea",
                "highway",
                "motorway_junction",
                "ref",
                "433B",
                "KindredCoda",
                "436",
                "mjn",
                "teodorab_telenav",
                "Adamant1",
                "amenity",
                "bank",
                "brand",
                "Chase",
                "brand:wikidata",
                "Q524629",
                "brand:wikipedia"
            ]
        );

        assert_eq!(
            &strs[(delimeters.len() - 10)..],
            vec![
                "Ox Fire Road",
                "Mill Pond Trail",
                "Skyline to the Sea - Saratoga Toll Road Connector Tail",
                "Auchmar Trail",
                "Ridge Top Trail",
                "site",
                "120.86",
                "124.18",
                "La Barranca Park",
                "Approximated, quite synthetic"
            ]
        );
    }

    #[test]
    fn test_build_rules_santacruz() {
        let manifest = manifest::parse(None);
        let archive = Osm::open(FileResourceStorage::new("tests/fixtures/santacruz/sort")).unwrap();
        let rules = Rules::new(&manifest, &archive);

        // boundary = administrative
        let mut tag = Tag::new();
        tag.set_key_idx(28381);
        tag.set_value_idx(6223);

        let zr = 0..12;
        let zoom_range = rules.get_zoom_range(&tag);
        assert_eq!(zoom_range, ZoomRangeRuleEval::Value(&zr));

        // key of building
        let mut tag2 = Tag::new();
        tag2.set_key_idx(2716);
        tag2.set_value_idx(1);

        let zr2 = 10..12;
        let zoom_range2 = rules.get_zoom_range(&tag2);
        assert_eq!(zoom_range2, ZoomRangeRuleEval::Key(&zr2));
    }

    #[test]
    #[ignore]
    fn test_california_time() {
        let archive = Osm::open(FileResourceStorage::new(
            "/Users/n/geodata/flatdata/california",
        ))
        .unwrap();
        let strings: RawData = archive.stringtable();

        let time = Instant::now();
        let strs = get_strs(strings);
        assert_eq!(strs.len(), 7428597);
        // Total Time: 569ms 658us 875ns
        println!("Total Time: {}", format_duration(time.elapsed()));
    }
}
