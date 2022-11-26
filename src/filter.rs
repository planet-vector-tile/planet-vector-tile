use std::ops::Range;

use crate::{
    manifest::Manifest,
    osmflat::osmflat_generated::osm::{Node, Osm, Way},
    rules::{Rules, ZoomRangeRuleEval},
};

pub struct Filter<'a> {
    archive: &'a Osm,
    rules: Rules,
    leaf_zoom: u8,
}

impl<'a> Filter<'a> {
    pub fn new(manifest: &'a Manifest, archive: &'a Osm) -> Filter<'a> {
        Filter {
            archive,
            rules: Rules::new(manifest, archive),
            leaf_zoom: manifest.render.leaf_zoom,
        }
    }

    // https://stackoverflow.com/questions/25445761/returning-a-closure-from-a-function
    pub fn node_at_zoom(&self, zoom: u8) -> impl Fn(&&'a Node) -> bool + '_ {
        let ways = self.archive.ways();
        let relations = self.archive.relations();
        let tags_index = self.archive.tags_index();

        let evaluate_node = move |node: &&'a Node| -> bool {
            let range = node.tags();
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

    pub fn way_at_zoom(&self, zoom: u8) -> impl Fn(&&'a Way) -> bool + '_ {
        let relations = self.archive.relations();
        let tags_index = self.archive.tags_index();

        let evaluate_way = move |way: &&'a Way| -> bool {
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
        let tags_index = self.archive.tags_index();
        let tags = self.archive.tags();
        let mut winning_eval = ZoomRangeRuleEval::None;

        for i in &tags_index[tags_idx_range] {
            let tag = &tags[i.value() as usize];

            let eval = self.rules.get_zoom_range(tag);

            match winning_eval {
                ZoomRangeRuleEval::None => {
                    winning_eval = eval;
                }
                ZoomRangeRuleEval::Tag(_) => {
                    break;
                }
                ZoomRangeRuleEval::Value(_) => match eval {
                    ZoomRangeRuleEval::None => (),
                    ZoomRangeRuleEval::Tag(_) => {
                        winning_eval = eval;
                        break;
                    }
                    ZoomRangeRuleEval::Value(_) => (),
                    ZoomRangeRuleEval::Key(_) => (),
                },
                ZoomRangeRuleEval::Key(_) => match eval {
                    ZoomRangeRuleEval::None => (),
                    ZoomRangeRuleEval::Tag(_) => {
                        winning_eval = eval;
                        break;
                    }
                    ZoomRangeRuleEval::Value(_) => {
                        winning_eval = eval;
                    }
                    ZoomRangeRuleEval::Key(_) => (),
                },
            }
        }

        let default_range = self.leaf_zoom..self.leaf_zoom;

        let range = match winning_eval {
            ZoomRangeRuleEval::None => &default_range,
            ZoomRangeRuleEval::Tag(r) => r,
            ZoomRangeRuleEval::Value(r) => r,
            ZoomRangeRuleEval::Key(r) => r,
        };

        if zoom >= range.start && zoom <= range.end {
            true
        } else {
            false
        }
    }
}
