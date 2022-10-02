#![deny(clippy::all)]

extern crate planet_core;

#[macro_use]
extern crate napi_derive;

use planet_core::tile::Tile;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn info_tile(z: u8, x: u32, y: u32) -> String {
    let t = Tile::from_zxy(z, x, y);
    let s = format!("hello tile z: {}, x: {},  y: {}, h: {}", t.z, t.x, t.y, t.h);
    s
}
