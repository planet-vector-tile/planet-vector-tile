mod args;
mod hilbert;
pub mod info_tile;
mod mutant;
mod osmflat;
mod parallel;
mod sort_archive;
pub mod tile;
pub mod tile_attributes;
mod util;

#[macro_use]
extern crate napi_derive;

use std::path::PathBuf;
use std::error::Error;
use args::Args;
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

// NHTODO This removes all the dead code warnings, because lib is the main codepath, not the pvt main bin.
// We should expose the CLI as a NodeJS binding here anyway.
// Don't use this, not finished implementing...
#[napi]
pub async fn pvt() -> Result<()> {
    let args = Args {
        input: PathBuf::from("./test/fixtures/4nodes/4nodes.osm.pbf"),
        output: PathBuf::from("./test/fixtures/4nodes"),
        ids: false,
        overwrite: false,
        leafzoom: 12,
    };
    let dir = args.output.clone();
    let archive = osmflat::convert(&args).unwrap_or_else(quit);
    sort_archive::sort(archive, &dir).unwrap_or_else(quit);
    hilbert::HilbertTiles::build(&dir, args.leafzoom).unwrap_or_else(quit);
    Ok(())
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
