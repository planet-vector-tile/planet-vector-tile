use std::ops::Range;

use super::{
    leaf::Leaf,
    tree::{FindResult, ResultPair},
};
use crate::tile::planet_vector_tile_generated::*;
use flatdata::RawData;

use crate::{
    hilbert::hilbert_tile::HilbertTile,
    hilbert::tree::HilbertTree,
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
    fn compose_leaf(&self, tile: &Tile, pair: ResultPair<&Leaf>, builder: &mut PVTBuilder) {
        let nodes = self.flatdata.nodes();
        let ways = self.flatdata.ways();
        let relations = self.flatdata.relations();
        let nodes_len = nodes.len();
        let ways_len = ways.len();
        let relations_len = relations.len();
        let external_entities = self.leaves_external.slice();

        // The range of indices in the entities vectors.
        let (n_range, w_range, _r_range, w_ext_range) = if let Some(next) = pair.next {
            (
                (pair.item.n as usize)..(next.n as usize),
                (pair.item.w as usize)..(next.w as usize),
                (pair.item.r as usize)..(next.r as usize),
                (pair.item.w_ext as usize)..(next.w_ext as usize),
            )
        } else {
            (
                (pair.item.n as usize)..nodes_len,
                (pair.item.w as usize)..ways_len,
                (pair.item.r as usize)..relations_len,
                (pair.item.w_ext as usize)..external_entities.len(),
            )
        };

        let ways_ext = w_ext_range.map(|i| external_entities[i] as usize);
        let ways_it = w_range.chain(ways_ext);
        self.build_pvt(n_range, ways_it, tile, builder)
    }

    fn compose_h_tile(
        &self,
        tile: &Tile,
        pair: ResultPair<&HilbertTile>,
        builder: &mut PVTBuilder,
    ) {
        let tile_n_idx = self.n.slice();
        let tile_w_idx = self.w.slice();

        let (n_range, w_range, _r_range) = if let Some(next) = pair.next {
            (
                (pair.item.n as usize)..(next.n as usize),
                (pair.item.w as usize)..(next.w as usize),
                (pair.item.r as usize)..(next.r as usize),
            )
        } else {
            (
                (pair.item.n as usize)..self.n.len,
                (pair.item.w as usize)..self.w.len,
                (pair.item.r as usize)..self.r.len,
            )
        };

        let nodes_it = n_range.map(|i| tile_n_idx[i] as usize).into_iter();
        let ways_it = w_range.map(|i| tile_w_idx[i] as usize).into_iter();

        self.build_pvt(nodes_it, ways_it, tile, builder)
    }

    fn build_pvt<N, W>(&self, nodes_it: N, ways_it: W, tile: &Tile, builder: &mut PVTBuilder)
    where
        N: Iterator<Item = usize>,
        W: Iterator<Item = usize>,
    {
        let nodes = self.flatdata.nodes();
        let ways = self.flatdata.ways();
        let relations = self.flatdata.relations();
        let nodes_len = nodes.len();
        let ways_len = ways.len();
        let relations_len = relations.len();
        let node_pairs = self.flatdata.hilbert_node_pairs().unwrap();
        let tags = self.flatdata.tags();
        let nodes_index = self.flatdata.nodes_index();
        let nodes_index_len = nodes_index.len();
        let tags_index = self.flatdata.tags_index();
        let tags_index_len = tags_index.len();
        let strings = self.flatdata.stringtable();

        // We reuse this for nodes, ways, relations.
        let mut features = Vec::new();

        for i in nodes_it {
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

        for i in ways_it {
            let way = &ways[i];

            let range = way.tags();
            let tags_index_start = range.start as usize;
            let tags_index_end = if range.end != 0 {
                range.end as usize
            } else {
                if relations_len > 0 {
                    relations[0].tag_first_idx() as usize
                } else {
                    tags_index_len
                }
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
            let range = way.refs();
            let refs_index_start = range.start as usize;
            let refs_index_end = if range.end != 0 {
                range.end as usize
            } else {
                nodes_index_len
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
                    id: way.osm_id() as u64, // NHTODO get h instead of osm_id
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
    }
}

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
        let k = unsafe { strings.substring_unchecked(tag.key_idx() as usize) };
        let v = unsafe { strings.substring_unchecked(tag.value_idx() as usize) };
        keys.push(builder.attributes.upsert_string(k));
        vals.push(builder.attributes.upsert_string_value(v));
    }

    (keys, vals)
}

#[cfg(test)]
mod tests {
    use crate::manifest;

    use super::*;

    #[test]
    fn test_basic_compose_tile() {
        // Scotts Valley
        // 9, 659, 1593
        let t = Tile::from_zh(12, 3329134);

        let manifest = manifest::parse("tests/fixtures/santacruz_sort.yaml").unwrap();
        let tree = HilbertTree::open(&manifest).unwrap();

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
        assert_eq!(features.len(), 2748);

        let feature = features.get(0);

        let id = feature.id();
        assert_eq!(id, 3660421543731798272); // hilbert location of node

        let keys = feature.keys().unwrap();
        assert_eq!(keys.len(), 2);
        let vals = feature.values().unwrap();

        let pvt_values = pvt.values().unwrap();

        let k0 = strings.get(keys.get(0) as usize);
        let v0 = pvt_values.get(vals.get(0) as usize).v();
        assert_eq!(k0, "osm_id");
        assert_eq!(v0, 5680698655.0);

        let k1 = strings.get(keys.get(1) as usize);
        let v1 = strings.get(pvt_values.get(vals.get(1) as usize).v() as usize);
        assert_eq!(k1, "power");
        assert_eq!(v1, "pole");

        let geometries = feature.geometries().unwrap();
        let len = geometries.len();
        assert_eq!(len, 1);
        let points = geometries.get(0).points().unwrap();
        let len = points.len();
        assert_eq!(len, 1);
        let point = points.get(0);
        let x = point.x();
        let y = point.y();
        assert_eq!(x, 162);
        assert_eq!(y, 58);
    }

    #[test]
    fn test_tags_index() {
        let manifest = manifest::parse("tests/fixtures/santacruz_sort.yaml").unwrap();
        let tree = HilbertTree::open(&manifest).unwrap();
        let nodes = tree.flatdata.nodes();
        for n in nodes {
            let t_range = n.tags();
            assert!(t_range.start <= t_range.end || t_range.end == 0);
        }
    }

    #[test]
    fn test_h_tile() {
        let manifest = manifest::parse("tests/fixtures/santacruz_sort.yaml").unwrap();
        let tree = HilbertTree::open(&manifest).unwrap();

        let mut builder = PVTBuilder::new();
        let t = Tile::from_zh(2, 3);
        tree.compose_tile(&t, &mut builder);
        let vec_u8 = builder.build();
        // just making sure no panic happened and there is content
        assert!(vec_u8.len() > 1000);
    }
}
