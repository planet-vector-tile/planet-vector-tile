use flatbuffers::FlatBufferBuilder;
use flatbuffers::WIPOffset;

use crate::pvt_builder::PVTBuilder;
use crate::source::Source;
use crate::tile::planet_vector_tile_generated::*;
use crate::tile::{HilbertBearing, Tile};

pub struct Info {
    grandchild_levels: u8,
}

impl Info {
    pub fn new() -> Self {
        Info { grandchild_levels: 4 }
    }
}

impl Source for Info {
    fn compose_tile(&self, tile: &Tile, builder: &mut PVTBuilder) {
        info(tile, builder, self.grandchild_levels)
    }
}

fn info(render_tile: &Tile, builder: &mut PVTBuilder, grandchild_levels: u8) {
    let pyramid = render_tile.pyramid(grandchild_levels);

    let mut boundary_vec = Vec::<WIPOffset<PVTFeature>>::new();
    let mut center_vec = Vec::<WIPOffset<PVTFeature>>::new();
    let mut bearing_vec = Vec::<WIPOffset<PVTFeature>>::new();

    for tile in &pyramid {
        let (boundary, center, bearing) = generate_features(render_tile, tile, builder, &pyramid);
        boundary_vec.push(boundary);
        center_vec.push(center);
        bearing_vec.push(bearing);
    }

    let fbb = &mut builder.fbb;
    let attributes = &mut builder.attributes;

    let boundary_features = fbb.create_vector(&boundary_vec);
    let boundary_layer = PVTLayer::create(
        fbb,
        &PVTLayerArgs {
            name: attributes.upsert_string("tile_boundary"),
            features: Some(boundary_features),
        },
    );
    let center_features = fbb.create_vector(&center_vec);
    let center_layer = PVTLayer::create(
        fbb,
        &PVTLayerArgs {
            name: attributes.upsert_string("tile_center"),
            features: Some(center_features),
        },
    );
    let bearing_features = fbb.create_vector(&bearing_vec);
    let bearing_layer = PVTLayer::create(
        fbb,
        &PVTLayerArgs {
            name: attributes.upsert_string("tile_bearing"),
            features: Some(bearing_features),
        },
    );

    builder.add_layer(boundary_layer);
    builder.add_layer(center_layer);
    builder.add_layer(bearing_layer);
}

pub fn generate_features<'a>(
    render_tile: &Tile,
    tile: &Tile,
    builder: &mut PVTBuilder<'a>,
    pyramid: &Vec<Tile>,
) -> (
    WIPOffset<PVTFeature<'a>>,
    WIPOffset<PVTFeature<'a>>,
    WIPOffset<PVTFeature<'a>>,
) {
    let fbb = &mut builder.fbb;
    let attributes = &mut builder.attributes;

    let id = tile.id();
    let is_render_tile = if render_tile == tile { 1_f64 } else { 0_f64 };
    let is_highest_zoom = match pyramid.last() {
        Some(t) => {
            if t.z == tile.z {
                1_f64
            } else {
                0_f64
            }
        }
        None => 0_f64,
    };

    // Create tags for features
    let tile_key = attributes.upsert_string("tile");
    let render_tile_key = attributes.upsert_string("render_tile");
    let is_render_tile_key = attributes.upsert_string("is_render_tile");
    let is_highest_zoom_key = attributes.upsert_string("highest_zoom");

    let z_key = attributes.upsert_string("z");
    let x_key = attributes.upsert_string("x");
    let y_key = attributes.upsert_string("y");
    let h_key = attributes.upsert_string("h");
    let o_key = attributes.upsert_string("o");

    let tile_val = attributes.upsert_string_value(&tile.to_string());
    let render_tile_val = attributes.upsert_string_value(&tile.to_string());
    let o_val = attributes.upsert_string_value(format!("{:?}", &tile.origin_float()).as_str());
    let is_render_tile_val =
        attributes.upsert_value(PVTValue::new(PVTValueType::Boolean, is_render_tile));
    let is_highest_zoom_val =
        attributes.upsert_value(PVTValue::new(PVTValueType::Boolean, is_highest_zoom));

    let z_val = attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.z as f64));
    let x_val = attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.x as f64));
    let y_val = attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.y as f64));
    let h_val = attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.h as f64));

    let keys = fbb.create_vector::<u32>(&[
        tile_key,
        render_tile_key,
        is_render_tile_key,
        is_highest_zoom_key,
        z_key,
        x_key,
        y_key,
        h_key,
        o_key,
    ]);
    let vals = fbb.create_vector::<u32>(&[
        tile_val,
        render_tile_val,
        is_render_tile_val,
        is_highest_zoom_val,
        z_val,
        x_val,
        y_val,
        h_val,
        o_val,
    ]);

    // Create boundary geometry
    let bbox = tile.bbox();

    let nw = render_tile.project(bbox.nw());
    let sw = render_tile.project(bbox.sw());
    let se = render_tile.project(bbox.se());
    let ne = render_tile.project(bbox.ne());

    let path = fbb.create_vector(&[nw, sw, se, ne, nw]);
    let geometry = PVTGeometry::create(fbb, &PVTGeometryArgs { points: Some(path) });
    let geometries = fbb.create_vector(&[geometry]);

    // Create boundary feature
    let boundary_feature = PVTFeature::create(
        fbb,
        &PVTFeatureArgs {
            id,
            keys: Some(keys),
            values: Some(vals),
            geometries: Some(geometries),
        },
    );

    // Create center geometry
    let center = render_tile.project(tile.center());
    let center_path = fbb.create_vector(&[center]);
    let center_geom = PVTGeometry::create(
        fbb,
        &PVTGeometryArgs {
            points: Some(center_path),
        },
    );
    let center_geoms = fbb.create_vector(&[center_geom]);

    // Create center feature.
    let center_feature = PVTFeature::create(
        fbb,
        &PVTFeatureArgs {
            id,
            keys: Some(keys),
            values: Some(vals),
            geometries: Some(center_geoms),
        },
    );

    let bearing_tile_points = create_bearing_tile_points(&render_tile, &tile);
    let bearing_path = fbb.create_vector(&bearing_tile_points);
    let bearing_geom = PVTGeometry::create(
        fbb,
        &PVTGeometryArgs {
            points: Some(bearing_path),
        },
    );
    let bearing_geoms = fbb.create_vector(&[bearing_geom]);

    let bearing_feature = PVTFeature::create(
        fbb,
        &PVTFeatureArgs {
            id,
            keys: Some(keys),
            values: Some(vals),
            geometries: Some(bearing_geoms),
        },
    );

    (boundary_feature, center_feature, bearing_feature)
}

fn create_bearing_tile_points(render_tile: &Tile, tile: &Tile) -> Vec<PVTTilePoint> {
    let origin = tile.origin_location();
    let extent = tile.location_extent();
    let middle = extent >> 1;

    let n = (origin.0 + middle, origin.1);
    let w = (origin.0, origin.1 + middle);
    let s = (origin.0 + middle, origin.1 + extent);
    let e = (origin.0 + extent, origin.1 + middle);

    let pn = render_tile.project(n);
    let pw = render_tile.project(w);
    let ps = render_tile.project(s);
    let pe = render_tile.project(e);
    let pc = render_tile.project(tile.center());

    match tile.hilbert_bearing() {
        HilbertBearing::NW => {
            vec![pn, pc, pw]
        }
        HilbertBearing::NS => {
            vec![pn, pc, ps]
        }
        HilbertBearing::NE => {
            vec![pn, pc, pe]
        }
        HilbertBearing::WS => {
            vec![pw, pc, ps]
        }
        HilbertBearing::WE => {
            vec![pw, pc, pe]
        }
        HilbertBearing::WN => {
            vec![pw, pc, pn]
        }
        HilbertBearing::SE => {
            vec![ps, pc, pe]
        }
        HilbertBearing::SN => {
            vec![ps, pc, pn]
        }
        HilbertBearing::SW => {
            vec![ps, pc, pw]
        }
        HilbertBearing::EN => {
            vec![pe, pc, pn]
        }
        HilbertBearing::EW => {
            vec![pe, pc, pw]
        }
        HilbertBearing::ES => {
            vec![pe, pc, ps]
        }
        HilbertBearing::None => {
            vec![]
        }
    }
}

pub fn basic(tile: Tile) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::with_capacity(1024);

    let nw = PVTTilePoint::new(0, 0);
    let sw = PVTTilePoint::new(0, 8192);
    let se = PVTTilePoint::new(8192, 8192);
    let ne = PVTTilePoint::new(8192, 0);

    let path = builder.create_vector(&[nw, sw, se, ne, nw]);

    let geometry = PVTGeometry::create(&mut builder, &PVTGeometryArgs { points: Some(path) });

    let keys = builder.create_vector::<u32>(&[0, 1, 2, 3]);
    let values = builder.create_vector::<u32>(&[0, 1, 2, 3]);
    let geometries = builder.create_vector(&[geometry]);

    let boundary_feature = PVTFeature::create(
        &mut builder,
        &PVTFeatureArgs {
            id: tile.h,
            keys: Some(keys),
            values: Some(values),
            geometries: Some(geometries),
        },
    );
    let boundary_features = builder.create_vector(&[boundary_feature]);
    let boundary_layer = PVTLayer::create(
        &mut builder,
        &PVTLayerArgs {
            name: 4,
            features: Some(boundary_features),
        },
    );

    let center_point = PVTTilePoint::new(4096, 4096);
    let center_path = builder.create_vector(&[center_point]);
    let center_geom = PVTGeometry::create(
        &mut builder,
        &PVTGeometryArgs {
            points: Some(center_path),
        },
    );
    let center_geoms = builder.create_vector(&[center_geom]);
    let center_feature = PVTFeature::create(
        &mut builder,
        &PVTFeatureArgs {
            id: tile.h,
            keys: Some(keys),
            values: Some(values),
            geometries: Some(center_geoms),
        },
    );
    let center_features = builder.create_vector(&[center_feature]);
    let center_layer = PVTLayer::create(
        &mut builder,
        &PVTLayerArgs {
            name: 5,
            features: Some(center_features),
        },
    );

    let layers = builder.create_vector(&[boundary_layer, center_layer]);

    let z = PVTValue::new(PVTValueType::Number, tile.z as f64);
    let x = PVTValue::new(PVTValueType::Number, tile.x as f64);
    let y = PVTValue::new(PVTValueType::Number, tile.y as f64);
    let h = PVTValue::new(PVTValueType::Number, tile.h as f64);

    let z_str = builder.create_string("z");
    let x_str = builder.create_string("x");
    let y_str = builder.create_string("y");
    let h_str = builder.create_string("h");
    let boundary_str = builder.create_string("tile_boundary");
    let center_str = builder.create_string("tile_center");

    let strings = builder.create_vector(&[z_str, x_str, y_str, h_str, boundary_str, center_str]);
    let values = builder.create_vector(&[z, x, y, h]);

    let tile = PVTTile::create(
        &mut builder,
        &PVTTileArgs {
            layers: Some(layers),
            strings: Some(strings),
            values: Some(values),
        },
    );

    builder.finish(tile, None);

    builder.finished_data().to_vec()
}

// 9, 82, 199
// let tile = Tile::from_zxy(z, x, y);
// let info_tile = InfoTile::new(tile, None);
// let vec_u8 = info_tile.build_buffer();
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_basic_info_tile() {
        let tile = Tile::from_zxy(9, 82, 199);
        let mut builder = PVTBuilder::new();
        info(&tile, &mut builder, 4);
        let vec_u8 = builder.build();

        assert!(vec_u8.len() > 100000);
    }

    #[test]
    fn test_zero_info_tile() {
        let tile = Tile::from_zxy(0, 0, 0);
        let mut builder = PVTBuilder::new();
        info(&tile, &mut builder, 4);
        let vec_u8 = builder.build();

        assert!(vec_u8.len() > 100000);
    }
}
