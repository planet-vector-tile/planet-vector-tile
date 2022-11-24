use crate::manifest::Manifest;


pub struct Filter {

}

impl Filter {
    pub fn new(manifest: &Manifest) -> Filter {
        Filter {

        }
    }

    // pub fn filter_entities_for_zoom(nodes: &[Node], ways: &[&Way], relations: &[&Relation], zoom: u8) -> (Vec<&Node>, Vec<&Way>, Vec<&Relation>) {
    //     let mut filtered_nodes = Vec::new();
    //     let mut filtered_ways = Vec::new();
    //     let mut filtered_relations = Vec::new();

    //     for node in nodes {
    //         if node.zoom <= zoom {
    //             filtered_nodes.push(node);
    //         }
    //     }

    //     for way in ways {
    //         if way.zoom <= zoom {
    //             filtered_ways.push(way);
    //         }
    //     }

    //     for relation in relations {
    //         if relation.zoom <= zoom {
    //             filtered_relations.push(relation);
    //         }
    //     }

    //     (filtered_nodes, filtered_ways, filtered_relations)
    // }

}
