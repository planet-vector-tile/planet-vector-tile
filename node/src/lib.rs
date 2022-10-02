#![deny(clippy::all)]

// use planet_core::tile::*;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn info_tile(z: u8, x: u32, y: u32) -> String {
  format!("hello world {} {} {}", z, x, y)
}
