use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::{
    tile::planet_vector_tile_generated::{PVTLayer, PVTTile, PVTTileArgs},
    tile_attributes::TileAttributes,
};

// NHTODO Make a pool, use the pool

pub struct PVTBuilder<'a> {
    pub fbb: FlatBufferBuilder<'a>,
    pub attributes: TileAttributes,
    layers: Vec<WIPOffset<PVTLayer<'a>>>,
}

impl<'a> PVTBuilder<'a> {
    pub fn new() -> Self {
        Self {
            fbb: FlatBufferBuilder::new(),
            attributes: TileAttributes::new(),
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: WIPOffset<PVTLayer<'a>>) {
        self.layers.push(layer);
    }

    pub fn build(&'a mut self) -> Vec<u8> {
        let strings = self.attributes.build_strings(&mut self.fbb);
        let values = self.attributes.build_values(&mut self.fbb);
        let layers = self.fbb.create_vector(&mut self.layers);
        let tile = PVTTile::create(
            &mut self.fbb,
            &PVTTileArgs {
                layers: Some(layers),
                strings: Some(strings),
                values: Some(values),
            },
        );
        self.fbb.finish(tile, None);
        self.fbb.finished_data().to_vec()
    }
}
