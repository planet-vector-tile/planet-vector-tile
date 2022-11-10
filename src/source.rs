use crate::tile::Tile;

pub trait Source {
    fn tile(&self, tile: &Tile) -> Vec<u8>;
}
