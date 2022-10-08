#![deny(clippy::all)]

pub mod tile;

#[macro_use]
extern crate napi_derive;

use tile::Tile;
use napi::bindgen_prelude::*;
use napi::tokio::{self};
use napi::tokio::time::{sleep, Duration};

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn info_tile(z: u8, x: u32, y: u32) -> String {
    let t = Tile::from_zxy(z, x, y);
    let s = format!("info tile z: {}, x: {},  y: {}, h: {}", t.z, t.x, t.y, t.h);
    s
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

  #[napi]
  pub async fn tile(&self, z: u8, x: u32, y: u32) -> Result<String> {
    let p = self.path.clone();
    tokio::task::spawn(async move { 
      let t = Tile::from_zxy(z, x, y);
      let s = format!("{} tile z: {}, x: {},  y: {}, h: {}", p, t.z, t.x, t.y, t.h);
      sleep(Duration::from_secs(1)).await;
      Ok(s)
    }).await.unwrap()
  
  }

  #[napi]
  pub async fn async_multi_two(arg: u32) -> Result<u32> {
    tokio::task::spawn(async move { Ok(arg * 2) })
      .await
      .unwrap()
  }
}
