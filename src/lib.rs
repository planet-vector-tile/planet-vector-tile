#![deny(clippy::all)]

pub mod tile;
pub mod tile_info;

#[macro_use]
extern crate napi_derive;

use tile::Tile;
use tile_info::tile_info;

use napi::bindgen_prelude::*;
use napi::tokio::{self};

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn load_planet(path: String, minzoom: u8, maxzoom: u8) -> Planet {
  Planet::new(path, minzoom, maxzoom)
}

#[napi]
pub struct Planet {
  path: String,
  minzoom: u8,
  maxzoom: u8
}

#[napi]
impl Planet {
  #[napi(constructor)]
  pub fn new(path: String, minzoom: u8, maxzoom: u8) -> Self {
    Self {
      path,
      minzoom,
      maxzoom
    }
  }

  // Try using AsyncTask to create a Buffer
  // https://napi.rs/docs/concepts/async-task
  // https://github.com/napi-rs/napi-rs/blob/main/examples/napi/src/task.rs
  // https://github.com/napi-rs/napi-rs/blob/a12bdc4359dfaff191d1fd124bc5b28e0d90f1bb/crates/napi/src/env.rs#L397
  #[napi]
  pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>> {
    let p = self.path.clone();
    tokio::task::spawn(async move { 
      let t = Tile::from_zxy(z, x, y);
      let buf = tile_info(t);
      Ok(buf)
    }).await.unwrap()
  
  }

  #[napi]
  pub async fn async_multi_two(arg: u32) -> Result<u32> {
    tokio::task::spawn(async move { Ok(arg * 2) })
      .await
      .unwrap()
  }
}
