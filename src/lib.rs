mod args;
mod hilbert;
mod hilbert_compose;
pub mod info;
pub mod location;
mod mutant;
mod osmflat;
mod parallel;
mod sort_archive;
mod source;
pub mod tile;
pub mod tile_attributes;

#[macro_use]
extern crate napi_derive;

use args::Args;
use info::*;
use source::Source;
use std::error::Error;
use std::path::PathBuf;
use tile::Tile;

use napi::bindgen_prelude::*;
use napi::tokio::{self};

#[napi]
pub fn load_planet(tiles: Vec<String>) -> Planet {
    Planet::new(tiles)
}

#[napi]
pub struct Planet {
    tiles: Vec<String>,
    sources: Vec<Box<dyn Source>>,
}

#[napi]
impl Planet {
    #[napi(constructor)]
    pub fn new(tiles: Vec<String>) -> Self {
        Self {
            tiles,
            sources: vec![Box::new(Info::new())],
        }
    }

    #[napi]
    pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Uint8Array> {
        let tiles = self.tiles.clone();
        let sources = self.sources.clone();
        tokio::task::spawn(async move {
            let tile = Tile::from_zxy(z, x, y);

            println!("tiles {:?}", tiles);
            for s in sources {
                println!("source {:?}", s);
            }

            let info = Info::new();
            let vec_u8 = info.tile(&tile);
            Ok(vec_u8.into())
        })
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
