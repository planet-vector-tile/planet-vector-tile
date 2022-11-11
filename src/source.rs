use dyn_clone::DynClone;
use std::fmt::Debug;

use crate::tile::Tile;

pub trait Source: Sync + DynClone + Debug + Send {
    fn tile(&self, tile: &Tile) -> Vec<u8>;
}

dyn_clone::clone_trait_object!(Source);
