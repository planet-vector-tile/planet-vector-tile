use flatbuffers::FlatBufferBuilder;

#[allow(dead_code, unused_imports)]
#[path = "./fbs/planet_vector_tile_generated.rs"]
mod planet_vector_tile_generated;

use planet_vector_tile_generated::*;

use crate::tile::Tile;

pub fn info(tile: Tile) -> Vec<u8> {}

fn tile(builder: &FlatBufferBuilder, tile: &Tile) {
    let bbox = tile.bbox();
    let center = tile.center();
    let proj = tile.project_bbox(&bbox);

    let nw = PVTPoint::new(proj.w, proj.n);
    let sw = PVTPoint::new(proj.w, proj.s);
    let se = PVTPoint::new(proj.e, proj.s);
    let ne = PVTPoint::new(proj.e, proj.n);

    let path = builder.create_vector(&[nw, sw, se, ne, nw]);
    let geometry = PVTGeometry::create(&mut builder, &PVTGeometryArgs { points: Some(path) });
}

pub fn tile_info(tile: Tile) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::with_capacity(1024);

    let nw = PVTPoint::new(0, 0);
    let sw = PVTPoint::new(0, 8192);
    let se = PVTPoint::new(8192, 8192);
    let ne = PVTPoint::new(8192, 0);

    let path = builder.create_vector(&[nw, sw, se, ne, nw]);

    let geometry = PVTGeometry::create(&mut builder, &PVTGeometryArgs { points: Some(path) });

    let keys = builder.create_vector::<u32>(&[0, 1, 2, 3]);
    let values = builder.create_vector::<u32>(&[0, 1, 2, 3]);
    let geometries = builder.create_vector(&[geometry]);

    let boundary_feature = PVTFeature::create(
        &mut builder,
        &PVTFeatureArgs {
            id: tile.h,
            h: tile.h,
            keys: Some(keys),
            values: Some(values),
            geometry: Some(geometries),
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

    let center_point = PVTPoint::new(4096, 4096);
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
            h: tile.h,
            keys: Some(keys),
            values: Some(values),
            geometry: Some(center_geoms),
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
