use std::ops::Range;

use dashmap::DashSet;

use crate::{
    manifest::Manifest,
    osmflat::osmflat_generated::osm::{Node, Osm, Way},
    rules::{RuleMatch, Rules},
};

pub struct Filter<'a> {
    flatdata: &'a Osm,
    rules: Rules,
    leaf_zoom: u8,
}

impl<'a> Filter<'a> {
    pub fn new(manifest: &'a Manifest, flatdata: &'a Osm) -> Filter<'a> {
        Filter {
            flatdata,
            rules: Rules::build(manifest, flatdata),
            leaf_zoom: manifest.render.leaf_zoom,
        }
    }

    // https://stackoverflow.com/questions/25445761/returning-a-closure-from-a-function
    pub fn node_at_zoom(&self, zoom: u8) -> impl Fn(&(usize, &'a Node)) -> bool + '_ {
        let ways = self.flatdata.ways();
        let relations = self.flatdata.relations();
        let tags_index = self.flatdata.tags_index();

        let evaluate_node = move |(_, node): &(usize, &'a Node)| -> bool {
            let range = node.tags();
            // Don't include nodes without tags.
            if range.start == range.end {
                return false;
            }
            let tags_index_start = range.start as usize;
            let tags_index_end = if range.end != 0 {
                range.end as usize
            } else if ways.len() > 0 {
                ways[0].tag_first_idx() as usize
            } else if relations.len() > 0 {
                relations[0].tag_first_idx() as usize
            } else {
                tags_index.len()
            };

            self.evaluate_tags(tags_index_start..tags_index_end, zoom)
        };

        evaluate_node
    }

    pub fn way_at_zoom(&self, zoom: u8) -> impl Fn(&(usize, &'a Way)) -> bool + '_ {
        let relations = self.flatdata.relations();
        let tags_index = self.flatdata.tags_index();
        let way_set: DashSet<usize> = DashSet::new();

        let evaluate_way = move |(i, way): &(usize, &'a Way)| -> bool {
            if way_set.contains(i) {
                return false;
            }
            way_set.insert(*i);

            let range = way.tags();
            let tags_index_start = range.start as usize;
            let tags_index_end = if range.end != 0 {
                range.end as usize
            } else if relations.len() > 0 {
                relations[0].tag_first_idx() as usize
            } else {
                tags_index.len()
            };

            self.evaluate_tags(tags_index_start..tags_index_end, zoom)
        };

        evaluate_way
    }

    fn evaluate_tags(&self, tags_idx_range: Range<usize>, zoom: u8) -> bool {
        let tags_index = self.flatdata.tags_index();
        let mut winning_match = RuleMatch::None;

        for i in &tags_index[tags_idx_range] {
            let rule_match = self.rules.evaluate(self.flatdata, i.value() as usize);

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
