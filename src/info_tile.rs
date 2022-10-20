use flatbuffers::WIPOffset;
use flatbuffers::{FlatBufferBuilder};

use crate::tile::planet_vector_tile_generated::*;
use crate::tile::Tile;
use crate::tile_attributes::TileAttributes;

pub struct InfoTile {
    tile: Tile,
    pyramid: Vec<Tile>,
    attributes: TileAttributes,
}

impl InfoTile {
    pub fn new(tile: Tile, child_levels: Option<u8>) -> Self {
        let levels = child_levels.unwrap_or(4);
        InfoTile { 
            tile,
            pyramid: tile.pyramid(levels),
            attributes: TileAttributes::new()
         }
    }

    pub fn build_buffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let mut boundary_vec = Vec::<WIPOffset<PVTFeature>>::new();
        let mut center_vec = Vec::<WIPOffset<PVTFeature>>::new();
        for tile in &self.pyramid {
            let (boundary, center) = self.generate_info(&mut builder, tile);
            boundary_vec.push(boundary);
            center_vec.push(center);
        }

        let boundary_features = builder.create_vector(&boundary_vec);
        let boundary_layer = PVTLayer::create(
            &mut builder,
            &PVTLayerArgs {
                name: self.attributes.upsert_string("tile_boundary"),
                features: Some(boundary_features),
            },
        );
        let center_features = builder.create_vector(&center_vec);
        let center_layer = PVTLayer::create(
            &mut builder,
            &PVTLayerArgs {
                name: self.attributes.upsert_string("tile_center"),
                features: Some(center_features),
            },
        );

        let layers = builder.create_vector(&[boundary_layer, center_layer]);
        let strings_vec = self.attributes.strings();
        // There should be a cleaner way of doing this...
        let strs_vec = strings_vec.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        let strings = builder.create_vector_of_strings(&strs_vec);
        let values = builder.create_vector(&self.attributes.values());

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

    fn generate_info<'a>(&self, builder: &mut FlatBufferBuilder<'a>, tile: &Tile) -> (WIPOffset<PVTFeature<'a>>, WIPOffset<PVTFeature<'a>>){
        let id = tile.id();
        let is_render_tile = if *tile == self.tile { 1_f64 } else { 0_f64 };
        
        // Create tags for features
        let tile_key = self.attributes.upsert_string("tile");
        let render_tile_key = self.attributes.upsert_string("render_tile");
        let is_render_tile_key = self.attributes.upsert_string("render_tile");

        let z_key = self.attributes.upsert_string("z");
        let x_key = self.attributes.upsert_string("x");
        let y_key = self.attributes.upsert_string("y");
        let h_key = self.attributes.upsert_string("h");

        let tile_val = self.attributes.upsert_string_value(&tile.to_string());
        let render_tile_val = self.attributes.upsert_string_value(&self.tile.to_string());
        let is_render_tile_val = self.attributes.upsert_value(PVTValue::new(PVTValueType::Boolean, is_render_tile));

        let z_val = self.attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.z as f64));
        let x_val = self.attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.x as f64));
        let y_val = self.attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.y as f64));
        let h_val = self.attributes.upsert_value(PVTValue::new(PVTValueType::Number, tile.h as f64));

        let keys = builder.create_vector::<u32>(&[tile_key, render_tile_key, is_render_tile_key, z_key, x_key, y_key, h_key]);
        let vals = builder.create_vector::<u32>(&[tile_val, render_tile_val, is_render_tile_val, z_val, x_val, y_val, h_val]);

        // Create boundary geometry
        let bbox = tile.bbox();

        // println!("bbox nw {:#?}", bbox.nw());
        // println!("bbox sw {:#?}", bbox.sw());
        // println!("bbox se {:#?}", bbox.se());
        // println!("bbox ne {:#?}", bbox.ne());
        let nw = self.tile.project(bbox.nw());
        let sw = self.tile.project(bbox.sw());
        let se = self.tile.project(bbox.se());
        let ne = self.tile.project(bbox.ne());
        // println!("proj nw {:#?}", nw);
        // println!("proj sw {:#?}", sw);
        // println!("proj se {:#?}", se);
        // println!("proj ne {:#?}", ne);

        let path = builder.create_vector(&[nw, sw, se, ne, nw]);
        let geometry = PVTGeometry::create(builder, &PVTGeometryArgs { points: Some(path) });
        let geometries = builder.create_vector(&[geometry]);

        // Create boundary feature
        let boundary_feature = PVTFeature::create(
            builder,
            &PVTFeatureArgs {
                id,
                h: tile.h,
                keys: Some(keys),
                values: Some(vals),
                geometry: Some(geometries),
            },
        );

        // Create center geometry
        let center = tile.center();
        let center_path = builder.create_vector(&[center]);
        let center_geom = PVTGeometry::create(builder, &PVTGeometryArgs { points: Some(center_path) });
        let center_geoms = builder.create_vector(&[center_geom]);

        // Create center feature.
        let center_feature = PVTFeature::create(
            builder,
            &PVTFeatureArgs {
                id,
                h: tile.h,
                keys: Some(keys),
                values: Some(vals),
                geometry: Some(center_geoms),
            },
        );

        (boundary_feature, center_feature)
    }
}

pub fn basic(tile: Tile) -> Vec<u8> {
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


// 9, 82, 199
// let tile = Tile::from_zxy(z, x, y);
// let info_tile = InfoTile::new(tile, None);
// let vec_u8 = info_tile.build_buffer();
mod tests {
    use super::*;

    #[test]
    fn test_basic_info_tile() {
        let tile = Tile::from_zxy(9, 82, 199);
        let info_tile = InfoTile::new(tile, None);
        let vec_u8 = info_tile.build_buffer();
        assert_eq!(vec_u8.len(), 106024);
    }
}
