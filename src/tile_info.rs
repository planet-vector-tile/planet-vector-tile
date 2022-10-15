use flatbuffers::FlatBufferBuilder;

#[allow(dead_code, unused_imports)]
#[path = "./fbs/planet_vector_tile_generated.rs"]
mod planet_vector_tile_generated;

use planet_vector_tile_generated::*;

use crate::tile::Tile;

pub fn tile_info(tile: Tile) -> Vec<u8> {
  let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);

  let nw = PVTPoint::new(0, 0);
  let sw = PVTPoint::new(0, 8192);
  let se = PVTPoint::new(8192, 8192);
  let ne = PVTPoint::new(8192, 0);

  let path = builder.create_vector(&[nw, sw, se, ne, nw]);

  let geometry = PVTGeometry::create(&mut builder, &PVTGeometryArgs { points: Some(path) });

  let keys = builder.create_vector::<u32>(&[0, 1, 2, 3]);
  let values = builder.create_vector::<u32>(&[0, 1, 2, 3]);
  let geometries = builder.create_vector(&[geometry]);

  let feature = PVTFeature::create(
    &mut builder,
    &PVTFeatureArgs {
      id: tile.h,
      h: tile.h,
      keys: Some(keys),
      values: Some(values),
      geometry: Some(geometries)
    },
  );

  let features = builder.create_vector(&[feature]);

  let layer = PVTLayer::create(
    &mut builder,
    &PVTLayerArgs {
      name: 4,
      features: Some(features),
    },
  );

  let layers = builder.create_vector(&[layer]);

  let z = PVTValue::new(PVTValueType::Number, tile.z as f64);
  let x = PVTValue::new(PVTValueType::Number, tile.x as f64);
  let y = PVTValue::new(PVTValueType::Number, tile.y as f64);
  let h = PVTValue::new(PVTValueType::Number, tile.h as f64);

  let z_str = builder.create_string("z");
  let x_str = builder.create_string("x");
  let y_str = builder.create_string("y");
  let h_str = builder.create_string("h");
  let layer_name_str = builder.create_string("tile_info");

  let strings = builder.create_vector(&[z_str, x_str, y_str, h_str, layer_name_str]);
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
