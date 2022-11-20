use std::ops::Range;

use crate::tile::planet_vector_tile_generated::*;
use flatbuffers::{Vector, WIPOffset};
use flatdata::RawData;

use crate::{
    hilbert::{HilbertTile, HilbertTree, Leaf},
    location::lonlat_to_xy,
    osmflat::osmflat_generated::osm::{Tag, TagIndex},
    pvt_builder::PVTBuilder,
    source::Source,
    tile::{
        planet_vector_tile_generated::{
            PVTFeature, PVTFeatureArgs, PVTGeometry, PVTGeometryArgs, PVTLayer, PVTLayerArgs,
        },
        Tile,
    },
};

struct ResultPair<T> {
    item: T,
    next: Option<T>,
}

enum FindResult<'a> {
    HilbertTile(ResultPair<&'a HilbertTile>),
    Leaf(ResultPair<&'a Leaf>),
    None,
}

impl Source for HilbertTree {
    fn compose_tile(&self, tile: &Tile, builder: &mut PVTBuilder) {
        match self.find(tile) {
            FindResult::HilbertTile(pair) => self.compose_h_tile(tile, pair, builder),
            FindResult::Leaf(pair) => self.compose_leaf(tile, pair, builder),
            FindResult::None => (),
        }
    }
}

impl HilbertTree {
    fn find(&self, tile: &Tile) -> FindResult {
        if tile.z > self.leaf_zoom {
            return FindResult::None;
        }

        let h_tiles = self.tiles.slice();
        let leaves = self.leaves.slice();
        let mut h_tile = h_tiles.last().unwrap();
        let mut leaf = None;

        let mut z = 2;
        let mut i = 0;
        while z < tile.z {
            let h = tile.h >> (2 * (tile.z - z));
            i = match child_index(h_tile, h) {
                Some(i) => i,
                None => return FindResult::None,
            };
            if z == self.leaf_zoom {
                leaf = Some(&leaves[i]);
                break;
            }
            h_tile = &h_tiles[i];
            z += 2;
        }

        if let Some(leaf) = leaf {
            let next_leaf = if i + 1 < leaves.len() {
                Some(&leaves[i + 1])
            } else {
                None
            };
            FindResult::Leaf(ResultPair {
                item: leaf,
                next: next_leaf,
            })
        } else {
            let next_h_tile = if i + 1 < h_tiles.len() {
                Some(&h_tiles[i + 1])
            } else {
                None
            };
            FindResult::HilbertTile(ResultPair {
                item: h_tile,
                next: next_h_tile,
            })
        }
    }

    fn compose_leaf(&self, tile: &Tile, pair: ResultPair<&Leaf>, builder: &mut PVTBuilder) {
        let nodes = self.archive.nodes();
        let ways = self.archive.ways();
        let relations = self.archive.relations();
        let nodes_len = nodes.len();
        let ways_len = ways.len();
        let relations_len = relations.len();
        let node_pairs = self.archive.hilbert_node_pairs().unwrap();
        let way_pairs = self.way_pairs.slice();
        let tags = self.archive.tags();
        let nodes_index = self.archive.nodes_index();
        let tags_index = self.archive.tags_index();
        let tags_index_len = tags_index.len();
        let strings = self.archive.stringtable();

        // The range of indices in the entities vectors.
        let (n_range, w_range, r_range) = if let Some(next) = pair.next {
            (
                (pair.item.n as usize)..(next.n as usize),
                (pair.item.w as usize)..(next.w as usize),
                (pair.item.r as usize)..(next.r as usize),
            )
        } else {
            (
                (pair.item.n as usize)..nodes_len,
                (pair.item.w as usize)..ways_len,
                (pair.item.r as usize)..relations_len,
            )
        };

        // We reuse this for nodes, ways, relations.
        let mut features = Vec::with_capacity(w_range.end - w_range.start);

        for i in n_range {
            let node = &nodes[i];

            let tags_index_start = node.tag_first_idx() as usize;
            let tags_index_end = if i + 1 < nodes_len {
                nodes[i + 1].tag_first_idx() as usize
            } else if ways_len > 0 {
                ways[0].tag_first_idx() as usize
            } else if relations_len > 0 {
                relations[0].tag_first_idx() as usize
            } else {
                tags_index_len
            };
            let tags_index_range = tags_index_start..tags_index_end;

            // We don't include untagged nodes.
            if tags_index_start == tags_index_end {
                continue;
            }

            // Tags
            let (keys, vals) = build_tags(
                tags_index_range,
                node.osm_id(),
                tags_index,
                tags,
                strings,
                builder,
            );
            let keys_vec = builder.fbb.create_vector(&keys);
            let vals_vec = builder.fbb.create_vector(&vals);

            // Geometries
            let lon = node.lon();
            let lat = node.lat();
            let xy = lonlat_to_xy((lon, lat));
            let tile_point = tile.project(xy);
            let points = builder.fbb.create_vector(&[tile_point]);
            let mut geom_builder = PVTGeometryBuilder::new(&mut builder.fbb);
            geom_builder.add_points(points);
            let geom = geom_builder.finish();
            let geoms = builder.fbb.create_vector(&[geom]);

            let feature = PVTFeature::create(
                &mut builder.fbb,
                &PVTFeatureArgs {
                    id: node_pairs[i].h(),
                    keys: Some(keys_vec),
                    values: Some(vals_vec),
                    geometries: Some(geoms),
                },
            );
            features.push(feature);
        }

        let features_vec = builder.fbb.create_vector(&features);
        let name = builder.attributes.upsert_string("nodes");
        let layer = PVTLayer::create(
            &mut builder.fbb,
            &PVTLayerArgs {
                name,
                features: Some(features_vec),
            },
        );
        builder.add_layer(layer);
        features.clear();

        for i in w_range {
            let way = &ways[i];

            let tags_index_start = way.tag_first_idx() as usize;
            let tags_index_end = if i < ways_len {
                ways[i + 1].tag_first_idx() as usize
            } else if relations_len > 0 {
                relations[0].tag_first_idx() as usize
            } else {
                tags_index_len
            };
            let tags_index_range = tags_index_start..tags_index_end;

            // Tags
            let (keys, vals) = build_tags(
                tags_index_range,
                way.osm_id(),
                tags_index,
                tags,
                strings,
                builder,
            );
            let keys_vec = builder.fbb.create_vector(&keys);
            let vals_vec = builder.fbb.create_vector(&vals);

            // Geometries
            let refs_index_start = way.ref_first_idx() as usize;
            let refs_index_end = if i < ways_len {
                ways[i + 1].ref_first_idx() as usize
            } else {
                tags_index_len
            };
            let mut path = Vec::with_capacity(refs_index_end - refs_index_start);
            for i in refs_index_start..refs_index_end {
                if let Some(r) = nodes_index[i].value() {
                    let n = &nodes[r as usize];
                    let lon = n.lon();
                    let lat = n.lat();
                    let xy = lonlat_to_xy((lon, lat));
                    let tile_point = tile.project(xy);
                    path.push(tile_point);
                }
            }
            let points = builder.fbb.create_vector(&path);
            let geom = PVTGeometry::create(
                &mut builder.fbb,
                &PVTGeometryArgs {
                    points: Some(points),
                },
            );
            let geoms = builder.fbb.create_vector(&[geom]);

            let feature = PVTFeature::create(
                &mut builder.fbb,
                &PVTFeatureArgs {
                    id: way_pairs[i].h(),
                    keys: Some(keys_vec),
                    values: Some(vals_vec),
                    geometries: Some(geoms),
                },
            );
            features.push(feature);
        }

        let features_vec = builder.fbb.create_vector(&features);

        let name = builder.attributes.upsert_string("ways");
        let layer = PVTLayer::create(
            &mut builder.fbb,
            &PVTLayerArgs {
                name,
                features: Some(features_vec),
            },
        );
        builder.add_layer(layer);

        // NHTODO Relations
    }

    fn compose_h_tile(
        &self,
        tile: &Tile,
        pair: ResultPair<&HilbertTile>,
        builder: &mut PVTBuilder,
    ) {
        //NHTODO - First, we need to populate chunks...
    }
}

fn child_index(h_tile: &HilbertTile, child_h: u64) -> Option<usize> {
    let child_pos = child_h & 0xf;
    let mask = h_tile.mask;
    if mask >> child_pos & 1 != 1 {
        return None;
    }
    let mut offset = 0;
    for i in 0..child_pos {
        offset += mask >> i & 1;
    }
    Some(h_tile.child as usize + offset as usize)
}

// NHTODO Not using this. Having issues with mutable lifetime references. This tag logic is hard coded right now.
fn build_tags(
    tags_index_range: Range<usize>,
    osm_id: i64,
    tags_index: &[TagIndex],
    tags: &[Tag],
    strings: RawData,
    builder: &mut PVTBuilder,
) -> (Vec<u32>, Vec<u32>) {
    let len = tags_index_range.end - tags_index_range.start + 1;
    let mut keys: Vec<u32> = Vec::with_capacity(len);
    let mut vals: Vec<u32> = Vec::with_capacity(len);

    let osm_id_key = builder.attributes.upsert_string("osm_id");
    let osm_id_val = builder.attributes.upsert_number_value(osm_id as f64);
    keys.push(osm_id_key);
    vals.push(osm_id_val);

    for tag_idx in &tags_index[tags_index_range] {
        let tag_i = tag_idx.value() as usize;
        debug_assert!(tag_i < tags.len());
        let tag = &tags[tag_i];
        let k = tag.key_idx() as usize;
        let v = tag.value_idx() as usize;
        match strings.substring(k) {
            Ok(key) => keys.push(builder.attributes.upsert_string(key)),
            Err(e) => eprintln!("Invalid tag key {:?}", e),
        }
        match strings.substring(v) {
            Ok(val) => vals.push(builder.attributes.upsert_string_value(val)),
            Err(e) => eprintln!("Invalid tag val {:?}", e),
        }
    }

    (keys, vals)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_basic_find() {
        // Scotts Valley
        // z 12 x 659 y 1593
        let t = Tile::from_zh(12, 3329134);

        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let tree = HilbertTree::open(&dir, 12).unwrap();

        let res = tree.find(&t);

        assert!(res.is_some());

        let res = res.unwrap();

        assert!(res.leaf.is_some());

        let leaf = res.leaf.unwrap();

        assert_eq!(leaf.n, 944454);
        assert_eq!(leaf.w, 106806);
        assert_eq!(leaf.r, 0);
        assert_eq!(leaf.h, 3329134);
    }

    #[test]
    fn test_basic_compose_tile() {
        // Scotts Valley
        // 9, 659, 1593
        let t = Tile::from_zh(12, 3329134);

        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let tree = HilbertTree::open(&dir, 12).unwrap();

        let mut builder = PVTBuilder::new();
        tree.compose_tile(&t, &mut builder);

        assert_eq!(builder.layers.len(), 2);

        let vec_u8 = builder.build();

        let pvt = root_as_pvttile(&vec_u8).unwrap();
        let layers = pvt.layers().unwrap();
        assert_eq!(layers.len(), 2);

        let layer_str_idx = layers.get(0).name();
        let strings = pvt.strings().unwrap();
        let layer_name = strings.get(layer_str_idx as usize);
        assert_eq!(layer_name, "nodes");

        let features = layers.get(0).features().unwrap();
        assert_eq!(features.len(), 16450);

        let feature = features.get(0);
        let keys = feature.keys().unwrap();
        assert_eq!(keys.len(), 3);
        let vals = feature.values().unwrap();

        let pvt_values = pvt.values().unwrap();

        let k1 = strings.get(keys.get(1) as usize);
        let v1 = strings.get(pvt_values.get(vals.get(1) as usize).v() as usize);

        let k2 = strings.get(keys.get(2) as usize);
        let v2 = strings.get(pvt_values.get(vals.get(2) as usize).v() as usize);
        assert_eq!(k1, "content");
        assert_eq!(v1, "water");
        assert_eq!(k2, "man_made");
        assert_eq!(v2, "storage_tank");

        let geometries = feature.geometries().unwrap();
        let len = geometries.len();
        assert_eq!(len, 1);
        let points = geometries.get(0).points().unwrap();
        let len = points.len();
        assert_eq!(len, 1);
        let point = points.get(0);
        assert_eq!(point.x(), 7779);
        assert_eq!(point.y(), -163);
    }

    #[test]
    fn test_tags_index() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let tree = HilbertTree::open(&dir, 12).unwrap();
        let nodes = tree.archive.nodes();
        for n in nodes {
            let t_range = n.tags();
            assert!(t_range.start <= t_range.end || t_range.end == 0);
        }
    }

    #[test]
    fn test_find_kings_village_road() {
        // Scotts Valley
        // z 12 x 659 y 1593
        let t = Tile::from_zh(12, 3329134);

        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
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
}
