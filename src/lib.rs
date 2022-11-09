mod args;
mod hilbert;
pub mod info_tile;
pub mod location;
mod mutant;
mod osmflat;
mod parallel;
mod sort_archive;
pub mod tile;
pub mod tile_attributes;

#[macro_use]
extern crate napi_derive;

use args::Args;
use info_tile::*;
use std::error::Error;
use std::path::PathBuf;
use tile::Tile;

use napi::bindgen_prelude::*;
use napi::tokio::{self};

#[napi]
pub fn load_planet(tiles: Vec<String>, minzoom: u8, maxzoom: u8) -> Planet {
    Planet::new(tiles, minzoom, maxzoom)
}

// NHTODO: Think about minzoom maxzoom
#[allow(dead_code)]
#[napi]
pub struct Planet {
    tiles: Vec<String>,
    minzoom: u8,
    maxzoom: u8,
}

#[napi]
impl Planet {
    #[napi(constructor)]
    pub fn new(tiles: Vec<String>, minzoom: u8, maxzoom: u8) -> Self {
        Self {
            tiles,
            minzoom,
            maxzoom,
        }
    }

    #[napi]
    pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Uint8Array> {
        let tiles = self.tiles.clone();
        tokio::task::spawn(async move {
            // println!("tiles {:?}", tiles);

            let tile = Tile::from_zxy(z, x, y);
            let vec_u8 = info_tile(tile, 4);
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
    hilbert::HilbertTree::build(&dir, args.leafzoom).unwrap_or_else(quit);
    Ok(())
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
