use flatbuffers::{FlatBufferBuilder, WIPOffset};
use itertools::Itertools;

use crate::{
    tile::planet_vector_tile_generated::{PVTLayer, PVTTile, PVTTileArgs, PVTValue},
    tile_attributes::TileAttributes,
};

// NHTODO Make a pool, use the pool

pub struct PVTBuilder<'a> {
    pub fbb: FlatBufferBuilder<'a>,
    pub attributes: TileAttributes,
    pub layers: Vec<WIPOffset<PVTLayer<'a>>>,
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
        let str_offsets = self
            .attributes
            .strings
            .iter()
            .map(|(k, _)| self.fbb.create_string(k.as_str()))
            .collect_vec();
        let strings = self.fbb.create_vector(&str_offsets);

        let vals = self.attributes.values.iter().map(|(v, _)| v).collect_vec();
        let values = self.fbb.create_vector(&vals);

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
