use crate::pvt::PVT2;
use crate::pvt_builder::PVTBuilder;
use crate::tile::Tile;

pub trait Source: Sync + Send {
    fn compose_tile(&self, tile: &Tile, builder: &mut PVTBuilder, p2: &mut PVT2);
}
