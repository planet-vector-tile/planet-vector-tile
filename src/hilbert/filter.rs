use ahash::AHashMap;

use crate::{manifest::Manifest, osmflat::osmflat_generated::osm::{Node, Osm}};


pub struct Filter<'a> {
    archive: &'a Osm,
    str_to_idx: AHashMap<&'a str, usize>,
}

impl<'a> Filter<'a> {
    pub fn new(manifest: &Manifest, archive: &'a Osm) -> Filter<'a> {

        let str_to_idx: AHashMap<&'a str, usize> = AHashMap::new();

        for (_, rule) in &manifest.rules {

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
