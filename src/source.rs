
use crate::tile::Tile;

pub trait Source: Sync + Send {
    fn tile(&self, tile: &Tile) -> Vec<u8>;
}

