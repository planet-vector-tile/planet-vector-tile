mod args;
mod hilbert;
pub mod info;
pub mod location;
mod mutant;
mod osmflat;
mod parallel;
mod pvt_builder;
mod sort_archive;
mod source;
pub mod tile;
pub mod tile_attributes;

#[macro_use]
extern crate napi_derive;

use args::Args;
use hilbert::tree::HilbertTree;
use info::*;
use pvt_builder::PVTBuilder;
use source::Source;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tile::Tile;

use napi::bindgen_prelude::*;
use napi::tokio::sync::RwLock;
use napi::tokio::{self};

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
                let path = PathBuf::from(tile);
                // NHTODO Build a way for HilbertTree and Info to read a TOML file to determine options.
                match HilbertTree::open(&path, 12) {
                    Ok(tree) => {
                        let box_tree = Box::new(tree) as Box<dyn Source>;
                        sources.push(box_tree);
                    }
                    Err(err) => {
                        eprintln!("Unable to open {:?} Skipping...", path);
                        eprintln!("{:?} Skipping...", err);
                    }
                }
            }
        }

        Self {
            tiles,
            sources: Arc::new(RwLock::new(sources)),
        }
    }

    #[napi]
    pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Uint8Array> {
        let time = Instant::now();
        let sources_rw = self.sources.clone();
        let task_result = tokio::task::spawn(async move {
            let tile = Tile::from_zxy(z, x, y);
            let mut builder = PVTBuilder::new();
            let sources = sources_rw.read().await;
            for i in 0..sources.len() {
                let source = sources.get(i).unwrap();
                source.compose_tile(&tile, &mut builder);
            }
            let vec_u8 = builder.build();
            Ok(vec_u8.into())
        })
        .await
        .unwrap();
        println!("{}/{}/{} {}ms.", z, x, y, time.elapsed().as_millis());
        task_result
    }
}

// NHTODO This removes all the dead code warnings, because lib is the main codepath, not the pvt main bin.
// We should expose the CLI as a NodeJS binding here anyway.
// Don't use this, not finished implementing...
#[napi]
pub async fn pvt() -> Result<()> {
    let args = Args {
        input: PathBuf::from("./tests/fixtures/nodes4.osm.pbf"),
        output: PathBuf::from("./tests/fixtures/nodes4/debug"),
        manifest: None,
        ids: false,
        overwrite: false,
    };
    let dir = args.output.clone();
    let archive = osmflat::convert(&args).unwrap_or_else(quit);
    sort_archive::sort(archive, &dir).unwrap_or_else(quit);
    hilbert::tree::HilbertTree::build(&dir, 12).unwrap_or_else(quit);
    Ok(())
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
