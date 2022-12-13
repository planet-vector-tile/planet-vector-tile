use dashmap::{DashMap, DashSet};
use flatdata::RawData;
use humantime::format_duration;
use itertools::Itertools;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, ops::Range};

use crate::{
    manifest::{IncludeTags, Manifest, Rule},
    osmflat::osmflat_generated::osm::Osm,
    util,
};

type Err = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Rules {
    pub tags: BTreeMap<usize, RuleEval>,
    pub values: BTreeMap<usize, RuleEval>,
    pub keys: BTreeMap<usize, RuleEval>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RuleEval {
    pub name: String,
    pub minzoom: u8,
    pub maxzoom: u8,
    pub include: IncludeTagIdxs,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IncludeTagIdxs {
    None,
    All,
    Keys(BTreeSet<usize>),
}

pub enum RuleMatch {
    None,
    Tag(RuleEval),
    Value(RuleEval),
    Key(RuleEval),
}

impl Rules {
    pub fn open(manifest: &Manifest) -> Result<Self, Err> {
        let path = manifest.data.planet.join("rules.yaml");
        let s = fs::read_to_string(path)?;
        let rules: Rules = serde_yaml::from_str(&s)?;
        Ok(rules)
    }

    pub fn build(manifest: &Manifest, flatdata: &Osm) -> Self {
        let strs: DashSet<&str> = DashSet::new();
        let kvs: DashSet<(&str, &str)> = DashSet::new();
        for (_, rule) in &manifest.rules {
            for (k, v) in &rule.tags {
                strs.insert(k);
                strs.insert(v);
                kvs.insert((k, v));
            }
            for v in &rule.values {
                strs.insert(v);
            }
            for k in &rule.keys {
                strs.insert(k);
            }
            if let Some(IncludeTags::Keys(keys)) = &rule.include {
                for k in keys {
                    strs.insert(k);
                }
            }
        }

        let str_to_idx: DashMap<&str, usize> = DashMap::new();
        let strings = flatdata.stringtable();

        let t = util::timer("Scanning stringtable for rule and include_tags strings...");
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

        let t = util::timer("Scanning tags for matching tag rules...");
        let tag_to_idx: DashMap<(&str, &str), usize> = DashMap::new();
        let _ = flatdata.tags().par_iter().enumerate().find_any(|(i, tag)| {
            let k = unsafe { strings.substring_unchecked(tag.key_idx() as usize) };
            let v = unsafe { strings.substring_unchecked(tag.value_idx() as usize) };
            let t = (k, v);
            if kvs.contains(&t) {
                tag_to_idx.insert(t, *i);
                kvs.remove(&t);
            }
            if kvs.is_empty() {
                true
            } else {
                false
            }
        });

        let mut include_keys = BTreeSet::<usize>::new();
        if let Some(IncludeTags::Keys(keys)) = &manifest.render.include_tags {
            for k in keys {
                if let Some(idx) = str_to_idx.get(k.as_str()) {
                    include_keys.insert(*idx);
                }
            }
        }
        
        println!("Finished in {}", format_duration(t.elapsed()));

        if strs.len() > 0 {
            println!("NOTICE: Not all rules and include_tags were matched to a string in the stringtable. Unmatched strings:\n{:?}", strs.iter().map(|k| *k ).collect_vec());
        }
        if kvs.len() > 0 {
            println!("NOTICE: Not all tag kv rules were matched to an existing tag. Unmatched tags:\n{:?}", kvs.iter().map(|tpl| *tpl).collect_vec());
        }
        println!(
            "Built pointers to strings from rules and include_tags in: {}",
            format_duration(t.elapsed())
        );

        let mut tags = BTreeMap::<usize, RuleEval>::new();
        let mut values = BTreeMap::<usize, RuleEval>::new();
        let mut keys = BTreeMap::<usize, RuleEval>::new();

        let leaf_zoom = manifest.render.leaf_zoom;

        for (rule_name, rule) in &manifest.rules {
            
            let include_idxs = if let Some(include) = rule.include {
                match include {
                    IncludeTags::None => IncludeTagIdxs::None,
                    IncludeTags::All => IncludeTagIdxs::All,
                    IncludeTags::Keys(key_strs) => {
                        let mut include_keys = BTreeSet::<usize>::new();
                        for k in &key_strs {
                            if let Some(idx) = str_to_idx.get(k.as_str()) {
                                include_keys.insert(*idx);
                            }
                        }
                        IncludeTagIdxs::Keys(include_keys)
                    },
                }
            } else {
                IncludeTagIdxs::None
            };

            for (k, v) in &rule.tags {
                if let Some(t_idx) = tag_to_idx.get(&(k, v)) {
                    tags.insert(*t_idx, RuleEval::new(rule, rule_name, leaf_zoom, include_idxs));
                }
            }
            for v in &rule.values {
                if let Some(v_idx) = str_to_idx.get(v.as_str()) {
                    values.insert(*v_idx, RuleEval::new(rule, rule_name, leaf_zoom, include_idxs));
                }
            }
            for k in &rule.keys {
                if let Some(k_idx) = str_to_idx.get(k.as_str()) {
                    keys.insert(*k_idx, RuleEval::new(rule, rule_name, leaf_zoom, include_idxs));
                }
            }

        }

        let rules = Rules {
            tags,
            values,
            keys,
        };

        let rules_path = manifest.data.planet.join("rules.yaml");
        let manifest_str = serde_yaml::to_string(&rules).expect("Rules should serialize");
        fs::write(&rules_path, manifest_str)
            .expect("Rules should be able to be written to planet dir");

        println!("Serialized rules to {}", rules_path.display());

        rules
    }

    pub fn evaluate_tags(&self, flatdata: &Osm, tags_idx_range: Range<usize>) -> RuleMatch {
        let tags_index = flatdata.tags_index();
        let mut winning_match = RuleMatch::None;

        for i in &tags_index[tags_idx_range] {
            let rule_match = self.evaluate_tag(flatdata, i.value() as usize);

            match winning_match {
                // Any match is better than none.
                RuleMatch::None => {
                    winning_match = rule_match;
                }
                // Only a tag match trumps a value match.
                RuleMatch::Value(_) => match rule_match {
                    RuleMatch::Tag(_) => {
                        winning_match = rule_match;
                        break; // The best match, we're done.
                    }
                    // First value wins
                    RuleMatch::Value(_) => (),
                    RuleMatch::Key(_) => (),
                    RuleMatch::None => (),
                },
                // A tag match or a value match trumps a key match
                RuleMatch::Key(_) => match rule_match {
                    RuleMatch::Tag(_) => {
                        winning_match = rule_match;
                        break;
                    }
                    RuleMatch::Value(_) => {
                        winning_match = rule_match;
                    }
                    // First key wins
                    RuleMatch::Key(_) => (),
                    RuleMatch::None => (),
                },
                // Shouldn't get here
                RuleMatch::Tag(_) => {
                    // eprintln!("Error: evaluate_tags logic error.");
                    break;
                }
            }
        }
        winning_match
    }

    pub fn evaluate_tag(&self, flatdata: &Osm, tag_i: usize) -> RuleMatch {
        if let Some(r) = self.tags.get(&tag_i) {
            return RuleMatch::Tag(r.clone());
        }

        let tag = &flatdata.tags()[tag_i];
        let value = tag.value_idx() as usize;
        if let Some(r) = self.values.get(&value) {
            return RuleMatch::Value(r.clone());
        }
        let key = tag.key_idx() as usize;
        if let Some(r) = self.keys.get(&key) {
            return RuleMatch::Key(r.clone());
        }
        RuleMatch::None
    }
}

impl RuleEval {
    pub fn new(rule: &Rule, rule_name: &String, leaf_zoom: u8, include: IncludeTagIdxs) -> Self {
        Self {
            name: rule_name.clone(),
            minzoom: rule.minzoom,
            maxzoom: match rule.maxzoom {
                Some(max) => max,
                None => leaf_zoom,
            },
            include,
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
