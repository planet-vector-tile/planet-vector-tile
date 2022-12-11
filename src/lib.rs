#![allow(dead_code)]

mod filter;
mod hilbert;
pub mod info;
pub mod location;
mod manifest;
mod mutant;
pub mod osmflat;
mod parallel;
mod pvt_builder;
mod rules;
mod sort;
mod source;
pub mod tile;
pub mod tile_attributes;
pub mod util;

#[macro_use]
extern crate napi_derive;

use hilbert::tree::HilbertTree;
use info::*;
use napi::bindgen_prelude::*;
use napi::tokio::sync::RwLock;
use napi::tokio::{self};
use pvt_builder::PVTBuilder;
use source::Source;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tile::Tile;

#[napi]
pub fn load_planet(tiles: Vec<String>) -> Planet {
    Planet::new(tiles)
}

#[napi]
pub struct Planet {
    tiles: Vec<String>,
    sources: Arc<RwLock<Vec<Box<dyn Source>>>>,
}

#[napi]
impl Planet {
    #[napi(constructor)]
    pub fn new(tiles: Vec<String>) -> Self {
        let mut sources = Vec::new();
        for tile in &tiles {
            if tile == "info" {
                let info = Box::new(Info::new()) as Box<dyn Source>;
                sources.push(info);
            } else {
                match manifest::parse(tile) {
                    Ok(manifest) => match HilbertTree::open(&manifest) {
                        Ok(tree) => {
                            let box_tree = Box::new(tree) as Box<dyn Source>;
                            sources.push(box_tree);
                        }
                        Err(err) => {
                            eprintln!("Unable to open {} Error: {:?}", tile, err);
                            eprintln!("Skipping {}", tile);
                        }
                    },
                    Err(e) => {
                        eprintln!("Unable to parse manifest at {} Error: {:?}", tile, e);
                        eprintln!("Skipping {}", tile);
                    }
                }
            }
        }

        Self {
            tiles,
            sources: Arc::new(RwLock::new(sources)),
        }
    }

    // To see stack trace:
    // RUST_BACKTRACE=full npm start

    #[napi]
    pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Uint8Array> {
        let time = Instant::now();
        let sources_rw = self.sources.clone();
        let tile = Tile::from_zxy(z, x, y);
        let task_handle = tokio::task::spawn(async move {
            let mut builder = PVTBuilder::new();
            let sources = sources_rw.read().await;
            for i in 0..sources.len() {
                let source = sources.get(i).unwrap();
                source.compose_tile(&tile, &mut builder);
            }
            let vec_u8 = builder.build();
            Ok(vec_u8.into())
        });
        match task_handle.await {
            Ok(result) => {
                println!(
                    "{:8} {}/{}/{} {} ms",
                    tile.h,
                    z,
                    x,
                    y,
                    time.elapsed().as_millis()
                );
                result
            }
            Err(err) => Err(napi::Error::new(
                napi::Status::GenericFailure,
                format!("{:8} {}/{}/{} Error: {:?}", tile.h, z, x, y, err),
            )),
        }
    }

    #[napi]
    pub fn abort(&self, z: u8, x: u32, y: u32) {
        // NHTODO Provide ability to abort task
        // https://github.com/cyb0124/abort-on-drop/blob/master/src/lib.rs
        println!("abort {}/{}/{}", z, x, y);
    }
}

// NHTODO This removes all the dead code warnings, because lib is the main codepath, not the pvt main bin.
// We should expose the CLI as a NodeJS binding here anyway.
// Don't use this, not finished implementing...
#[napi]
pub async fn pvt() -> Result<()> {
    let manifest = manifest::parse("manifests/basic.yaml").unwrap();
    let flatdata = osmflat::convert(&manifest).unwrap_or_else(quit);
    sort::sort_flatdata(flatdata, &manifest.data.planet).unwrap_or_else(quit);
    hilbert::tree::HilbertTree::new(&manifest).unwrap_or_else(quit);
    Ok(())
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
