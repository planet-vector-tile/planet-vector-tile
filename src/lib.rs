#![deny(clippy::all)]

pub mod info_tile;
pub mod tile;
pub mod tile_attributes;
mod hilbert;
mod mutant;
mod osmflat;

#[macro_use]
extern crate napi_derive;

use info_tile::*;
use tile::Tile;

use napi::bindgen_prelude::*;
use napi::tokio::{self};

#[napi]
pub fn load_planet(path: String, minzoom: u8, maxzoom: u8) -> Planet {
    Planet::new(path, minzoom, maxzoom)
}

// NHTODO: Think about minzoom maxzoom
#[allow(dead_code)]
#[napi]
pub struct Planet {
    path: String,
    minzoom: u8,
    maxzoom: u8,
}

#[napi]
impl Planet {
    #[napi(constructor)]
    pub fn new(path: String, minzoom: u8, maxzoom: u8) -> Self {
        Self {
            path,
            minzoom,
            maxzoom,
        }
    }

    #[napi]
    pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Uint8Array> {
        // let p = self.path.clone();
        tokio::task::spawn(async move {
            let tile = Tile::from_zxy(z, x, y);
            let info_tile = InfoTile::new(tile, None);
            let vec_u8 = info_tile.build_buffer();
            // let vec_u8 = basic(tile);
            Ok(vec_u8.into())
        })
        .await
        .unwrap()
    }

    #[napi]
    pub async fn async_multi_two(arg: u32) -> Result<u32> {
        tokio::task::spawn(async move { Ok(arg * 2) })
            .await
            .unwrap()
    }
}
