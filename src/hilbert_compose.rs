use std::ops::Range;

use crate::{
    hilbert::{HilbertTile, HilbertTree, Leaf},
    location::lonlat_to_xy,
    pvt_builder::PVTBuilder,
    source::Source,
    tile::{
        planet_vector_tile_generated::{
            PVTFeature, PVTFeatureArgs, PVTGeometry, PVTGeometryArgs, PVTLayer, PVTLayerArgs,
        },
        Tile,
    },
    tile_attributes::TileAttributes,
};

struct FindResult<'a> {
    h_tile: &'a HilbertTile,
    next_h_tile: Option<&'a HilbertTile>,
    leaf: Option<&'a Leaf>,
    next_leaf: Option<&'a Leaf>,
}

impl HilbertTree {
    fn find(&self, tile: &Tile) -> Option<FindResult> {
        if tile.z > self.leaf_zoom {
            return None;
        }

        println!("Finding {:?}", tile);

        let h_tiles = self.tiles.slice();
        let mut h_tile = h_tiles.last().unwrap();
        println!("root {:?}", h_tile);

        let mut z = 2;
        let mut i = 0;
        while z <= tile.z {
            let h = tile.h >> (2 * (tile.z - z));
            let child_pos = (h & 0xf) as i32;

            // If the tile does not have the child position in the mask,
            // then we don't have the tile.
            if h_tile.mask >> child_pos & 1 != 1 {
                return None;
            }

            i = (h_tile.child + child_pos) as usize;

            h_tile = &h_tiles[i];

            println!("i {} {:?}", i, h_tile);

            z += 2;
        }

        let next_h_tile = if i + 1 < h_tiles.len() {
            Some(&h_tiles[i + 1])
        } else {
            None
        };

        let mut leaf = None;
        let mut next_leaf = None;
        if h_tile.mask == 0 {
            let leaves = self.leaves.slice();
            leaf = Some(&leaves[i]);
            if i + 1 < leaves.len() {
                next_leaf = Some(&leaves[i + 1]);
            }
        }

        Some(FindResult {
            h_tile,
            next_h_tile: None,
            leaf,
            next_leaf: None,
        })
    }
}

impl Source for HilbertTree {
    fn compose_tile(&self, tile: &Tile, builder: &mut PVTBuilder) {
        let fbb = &mut builder.fbb;
        let attributes = &mut builder.attributes;

        // The tile exists in the index
        if let Some(res) = self.find(tile) {
            // It is a leaf tile
            if let Some(leaf) = res.leaf {
                println!("LEAF FOUND {:?}", leaf);

                let nodes = self.archive.nodes();
                let node_pairs = self.archive.hilbert_node_pairs().unwrap();
                let ways = self.archive.ways();
                // let way_pairs = self.archive.hilbert_way_pairs_not_used()
                let relations = self.archive.relations();
                let tags_index = self.archive.tags_index();
                let tags = self.archive.tags();
                let strings = self.archive.stringtable();

                let n_start = leaf.n as usize;
                let w_start = leaf.w as usize;
                let r_start = leaf.r as usize;

                let (n_end, w_end, r_end) = if let Some(next_leaf) = res.next_leaf {
                    (
                        next_leaf.n as usize,
                        next_leaf.w as usize,
                        next_leaf.r as usize,
                    )
                } else {
                    (nodes.len(), ways.len(), relations.len())
                };

                let n_slice = &nodes[n_start..n_end];
                let w_slice = &ways[w_start..w_end];

                let mut features = vec![];

                for (i, n) in n_slice.iter().enumerate() {
                    let t_range = n.tags();
                    let start = t_range.start as usize;
                    let mut end = t_range.end as usize;
                    let tags_index_len = tags_index.len();

                    if start == end {
                        continue;
                    }

                    if start == 0 {
                        println!("start 0");
                    }
                    if start > end {
                        println!("start < end");
                    }
                    if end == 0 {
                        println!("end 0");
                        continue;
                    }
                    if end > tags_index_len {
                        println!("end > tags_index.len()");
                        end = tags.len();
                    }
                    if start > tags_index_len {
                        println!("start > tags_index.len()");
                        continue;
                    }

                    debug_assert!(start <= end);
                    debug_assert!(end != 0);
                    debug_assert!(end <= tags_index_len);
                    debug_assert!(start <= tags_index_len);
                 
                    let n_tag_idxs = &tags_index[start..end];

                    let mut keys: Vec<u32> = vec![];
                    let mut vals: Vec<u32> = vec![];

                    for tag_idx in n_tag_idxs {
                        let tag_i = tag_idx.value() as usize;
                        debug_assert!(tag_i < tags.len());
                        let tag = &tags[tag_i];
                        let k = tag.key_idx();
                        let v = tag.value_idx();

                        // NHTODO Consider switching to substring_unchecked after confident.
                        let key = strings.substring(k as usize);
                        let val = strings.substring(v as usize);

                        if key.is_ok() && val.is_ok() {
                            let key = key.unwrap();
                            let val = val.unwrap();

                            keys.push(attributes.upsert_string(key));
                            vals.push(attributes.upsert_string(val));
                        } else {
                            eprintln!("Invalid tag key val {:?} {:?}", key.unwrap_err(), val.unwrap_err());
                        }
                    }

                    let lon = n.lon();
                    let lat = n.lat();
                    // println!("lon lat {} {}", lon, lat);
                    let xy = lonlat_to_xy((lon, lat));
                    let tile_point = tile.project(xy);
                    // println!("tile_point {:?}", tile_point);

                    let h = node_pairs[i].h();

                    let keys_vec = fbb.create_vector(&keys);
                    let vals_vec = fbb.create_vector(&vals);

                    let path = fbb.create_vector(&[tile_point]);
                    let geom = PVTGeometry::create(fbb, &PVTGeometryArgs { points: Some(path) });
                    let geoms = fbb.create_vector(&[geom]);

                    // NHTODO Get rid of the h field in PVTFeature. It's "pointless".
                    let feature = PVTFeature::create(
                        fbb,
                        &PVTFeatureArgs {
                            id: h,
                            h,
                            keys: Some(keys_vec),
                            values: Some(vals_vec),
                            geometries: Some(geoms),
                        },
                    );
                    features.push(feature);
                }

                let features = fbb.create_vector(&features);

                let name = attributes.upsert_string("nodes");
                let layer = PVTLayer::create(
                    fbb,
                    &PVTLayerArgs {
                        name,
                        features: Some(features),
                    },
                );

                builder.add_layer(layer);

                for w in w_slice {}
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_basic_find() {
        // Scotts Valley
        let t = Tile::from_zh(12, 3329134);

        let dir = PathBuf::from("/Users/n/geodata/flatdata/santacruz");
        let tree = HilbertTree::open(&dir, 12).unwrap();

        let res = tree.find(&t);

        assert!(res.is_some());

        let res = res.unwrap();

        assert!(res.leaf.is_some());

        let leaf = res.leaf.unwrap();

        assert_eq!(leaf.n, 944454);
        assert_eq!(leaf.w, 106806);
        assert_eq!(leaf.r, 0);
        assert_eq!(leaf.h, 3329135);
    }

    #[test]
    fn test_basic_compose_tile() {
        // Scotts Valley
        let t = Tile::from_zh(12, 3329134);

        let dir = PathBuf::from("/Users/n/geodata/flatdata/santacruz");
        let tree = HilbertTree::open(&dir, 12).unwrap();

        let mut builder = PVTBuilder::new();
        tree.compose_tile(&t, &mut builder);

        assert_eq!(builder.layers.len(), 1);
    }
}
