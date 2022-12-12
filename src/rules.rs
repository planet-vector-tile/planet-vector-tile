use ahash::AHashMap;
use dashmap::{DashMap, DashSet};
use flatdata::RawData;
use humantime::format_duration;
use itertools::Itertools;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, ops::Range, time::Instant};
use yaml_rust::{yaml, YamlEmitter};

use crate::{
    manifest::{IncludeTags, Manifest},
    osmflat::osmflat_generated::osm::{Osm, Tag},
    util,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Rules {
    pub tags: BTreeMap<usize, RuleEval>,
    pub values: BTreeMap<usize, RuleEval>,
    pub keys: BTreeMap<usize, RuleEval>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RuleEval {
    pub idx: usize,
    pub name: String,
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
}

pub enum RuleMatch {
    None,
    Tag(RuleEval),
    Value(RuleEval),
    Key(RuleEval),
}

impl Rules {
    pub fn build(manifest: &Manifest, flatdata: &Osm) -> Self {
        let strs: DashSet<&str> = DashSet::new();
        let kvs: DashSet<(&str, &str)> = DashSet::new();
        for (_, rule) in &manifest.rules {
            for (k, v) in &rule.tags {
                strs.insert(k);
                strs.insert(v);
            }
            for v in &rule.values {
                strs.insert(v);
            }
            for k in &rule.keys {
                strs.insert(k);
            }
        }
        if let Some(IncludeTags::Keys(keys)) = &manifest.render.include_tags {
            for k in keys {
                strs.insert(k);
            }
        }

        let str_to_idx: DashMap<&str, usize> = DashMap::new();
        let strings = flatdata.stringtable();

        let t = util::timer("Scanning stringtable for rule strings...");
        // Note: This is expensive, but better than constantly strcmp against rules during the build.
        let str_ranges = get_str_ranges(strings);

        let _ = str_ranges.par_iter().find_any(|r| {
            let bytes = &strings.as_bytes()[r.start..r.end];
            let s = unsafe { std::str::from_utf8_unchecked(bytes) };
            if strs.contains(s) {
                str_to_idx.insert(s, r.start);
                strs.remove(s);
            }
            // halt iterating when the set is empty
            if strs.is_empty() {
                true
            } else {
                false
            }
        });
        println!("Finished in {}", format_duration(t.elapsed()));

        let t = util::timer("Scanning tags table for matching tag rules...");
        let tag_to_idx: DashMap<(&str, &str), usize> = DashMap::new();
        let _ = flatdata.tags().par_iter().enumerate().find_any(|(i, tag)| {
            let k = unsafe { strings.substring_unchecked(tag.key_idx() as usize) };
            let v = unsafe { strings.substring_unchecked(tag.value_idx() as usize) };
            let t = (k, v);
            if kvs.contains(&t) {
                tag_to_idx.insert(t, *i);
                kvs.remove(&t);
            }
            true
        });

        if strs.len() > 0 {
            println!("NOTICE: Not all rules and include_tags were matched to a string in the stringtable. Unmatched strings:\n{:?}", strs);
        }
        println!(
            "Built pointers to strings from rules and include_tags in: {}",
            format_duration(t.elapsed())
        );

        let str_to_idx_path = manifest.data.planet.join("str_to_idx.yaml");
        println!("Saving string index to {}", str_to_idx_path.display());
        let mut yaml_hash = yaml::Hash::new();
        for ref_multi in str_to_idx.iter() {
            let (k, v) = ref_multi.pair();
            yaml_hash.insert(
                yaml::Yaml::String(k.to_string()),
                yaml::Yaml::Integer(*v as i64),
            );
        }
        let mut str_to_idx_yaml_str = String::new();
        let mut emitter = YamlEmitter::new(&mut str_to_idx_yaml_str);
        match emitter.dump(&yaml::Yaml::Hash(yaml_hash)) {
            Ok(_) => {
                if let Err(err) = fs::write(&str_to_idx_path, str_to_idx_yaml_str) {
                    eprintln!(
                        "Failed to write string index to file {} Err: {}",
                        str_to_idx_path.display(),
                        err
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to write string index. Err: {}", e);
            }
        }

        let mut tags = BTreeMap::<usize, RuleEval>::new();
        let mut values = BTreeMap::<usize, RuleEval>::new();
        let mut keys = BTreeMap::<usize, RuleEval>::new();

        for (rule_name, rule) in &manifest.rules {
            let zoom_range = if let Some(maxzoom) = rule.maxzoom {
                rule.minzoom..maxzoom
            } else {
                rule.minzoom..manifest.render.leaf_zoom
            };

            for (idx, (k, v)) in rule.tags.iter().enumerate() {
                let t_idx = match tag_to_idx.get(&(k, v)) {
                    Some(idx) => *idx,
                    None => break,
                };
                let r = RuleEval {
                    idx,
                    name: rule_name.clone(),
                    minzoom: rule.minzoom,
                    maxzoom: rule.maxzoom,
                };
                tags.insert(t_idx, r);
            }
            for (idx, v) in rule.values.iter().enumerate() {
                let v_idx = match str_to_idx.get(v.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                let r = RuleEval {
                    idx,
                    name: rule_name.clone(),
                    minzoom: rule.minzoom,
                    maxzoom: rule.maxzoom,
                };
                values.insert(v_idx, r);
            }
            for (idx, k) in rule.keys.iter().enumerate() {
                let k_idx = match str_to_idx.get(k.as_str()) {
                    Some(idx) => *idx,
                    None => {
                        break;
                    }
                };
                let r = RuleEval {
                    idx,
                    name: rule_name.clone(),
                    minzoom: rule.minzoom,
                    maxzoom: rule.maxzoom,
                };
                keys.insert(k_idx, r);
            }
        }

        Rules { tags, values, keys }
    }

    pub fn evaluate(&self, flatdata: &Osm, tag_i: usize) -> RuleMatch {
        if let Some(r) = self.tags.get(&tag_i) {
            return RuleMatch::Tag(r.clone());
        }

        let tag = flatdata.tags()[tag_i];
        let key = tag.key_idx() as usize;
        let value = tag.value_idx() as usize;
        if let Some(r) = self.values.get(&value) {
            return RuleMatch::Value(r.clone());
        }
        if let Some(r) = self.keys.get(&key) {
            return RuleMatch::Key(r.clone());
        }
        RuleMatch::None
    }

    pub fn evaluate_tags(&self, flatdata: &Osm, tags_idx_range: Range<usize>, zoom: u8) -> bool {
        let tags_index = flatdata.tags_index();
        let mut winning_match = RuleMatch::None;

        for i in &tags_index[tags_idx_range] {
            let rule_match = self.evaluate(flatdata, i.value() as usize);

            match winning_match {
                RuleMatch::None => {
                    winning_match = rule_match;
                }
                RuleMatch::Tag(_) => {
                    break;
                }
                RuleMatch::Value(_) => match rule_match {
                    RuleMatch::None => (),
                    RuleMatch::Tag(_) => {
                        winning_match = rule_match;
                        break;
                    }
                    RuleMatch::Value(_) => (),
                    RuleMatch::Key(_) => (),
                },
                RuleMatch::Key(_) => match rule_match {
                    RuleMatch::None => (),
                    RuleMatch::Tag(_) => {
                        winning_match = rule_match;
                        break;
                    }
                    RuleMatch::Value(_) => {
                        winning_match = rule_match;
                    }
                    RuleMatch::Key(_) => (),
                },
            }
        }

        let mut minzoom = self.leaf_zoom;
        let mut maxzoom = self.leaf_zoom;

        match winning_match {
            RuleMatch::None => {}
            RuleMatch::Tag(r) => {
                minzoom = r.minzoom;
                if let Some(max) = r.maxzoom {
                    maxzoom = max
                };
            }
            RuleMatch::Value(r) => {
                minzoom = r.minzoom;
                if let Some(max) = r.maxzoom {
                    maxzoom = max
                };
            },
            RuleMatch::Key(r) => {
                minzoom = r.minzoom;
                if let Some(max) = r.maxzoom {
                    maxzoom = max
                };
            },
        };

        if zoom >= minzoom && zoom <= maxzoom {
            true
        } else {
            false
        }
    }
}

fn get_str_null_delimeters(strings: RawData) -> Vec<usize> {
    let bytes = strings.as_bytes();
    let delimeters: Vec<usize> = bytes
        .par_iter()
        .enumerate()
        .filter_map(|(i, byte)| if *byte == 0 { Some(i) } else { None })
        .collect();

    delimeters
}

fn get_str_ranges(strings: RawData) -> Vec<Range<usize>> {
    let delimeters = get_str_null_delimeters(strings);
    let ranges: Vec<Range<usize>> = delimeters
        .par_iter()
        .enumerate()
        .map(|(i, delimeter)| {
            let start = if i == 0 { 0 } else { delimeters[i - 1] + 1 };
            let end = *delimeter;
            start..end
        })
        .collect();

    ranges
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use flatdata::FileResourceStorage;
    use humantime::format_duration;

    use crate::manifest;

    use super::*;

    fn get_strs(strings: RawData) -> Vec<&str> {
        let bytes = strings.as_bytes();
        let ranges = get_str_ranges(strings);
        let strs: Vec<&str> = ranges
            .par_iter()
            .map(|r| {
                let slice = &bytes[r.start..r.end];
                let str = unsafe { std::str::from_utf8_unchecked(slice) };
                str
            })
            .collect();

        strs
    }

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
    fn test_get_strs_santa_cruz() {
        let flatdata =
            Osm::open(FileResourceStorage::new("tests/fixtures/santa_cruz/sort")).unwrap();
        let strings: RawData = flatdata.stringtable();
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
    fn test_build_rules_santa_cruz() {
        let manifest = manifest::parse("tests/fixtures/santa_cruz_sort.yaml").unwrap();
        let flatdata =
            Osm::open(FileResourceStorage::new("tests/fixtures/santa_cruz/sort")).unwrap();
        let rules = Rules::build(&manifest, &flatdata);

        // boundary = administrative
        let mut tag = Tag::new();
        tag.set_key_idx(406551);
        tag.set_value_idx(90476);

        let zr = 0..12;
        let zoom_range = rules.get_zoom_range(&tag);
        assert_eq!(zoom_range, RuleMatch::Value(&zr));

        // key of building
        let mut tag2 = Tag::new();
        tag2.set_key_idx(32840);
        tag2.set_value_idx(1);

        let zr2 = 10..12;
        let zoom_range2 = rules.get_zoom_range(&tag2);
        assert_eq!(zoom_range2, RuleMatch::Key(&zr2));
    }

    #[test]
    #[ignore]
    fn test_california_time() {
        let flatdata = Osm::open(FileResourceStorage::new(
            "/Users/n/geodata/flatdata/california",
        ))
        .unwrap();
        let strings: RawData = flatdata.stringtable();

        let time = Instant::now();
        let strs = get_str_ranges(strings);
        assert_eq!(strs.len(), 7428597);
        // Total Time: 569ms 658us 875ns
        println!("Total Time: {}", format_duration(time.elapsed()));
    }
}
