
use crate::pvt_builder::PVTBuilder;
use crate::tile_attributes::TileAttributes;

use crate::tile::Tile;

pub trait Source: Sync + Send {
    fn build_tile(&self, tile: &Tile, builder: &mut PVTBuilder);
}

