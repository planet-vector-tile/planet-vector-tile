#![allow(dead_code)]

use std::f64::consts::PI;
use fast_hilbert::{xy2h, h2xy};

/// Projects dm7 lonlat to Web Mercator points in float space (0.0 -> 1.0).
fn project_lonlat_to_mercator(lonlat: (i32, i32)) -> (f64, f64) {
    let lon = lonlat.0 as f64 / 10_000_000f64;
    let lat = lonlat.1 as f64 / 10_000_000f64;
    let mut x = lon / 360.0 + 0.5;
    let mut y = lat * PI / 180.0;
    y = 0.5 - 0.25 * (1.0 + y.sin() / (1.0 - y.sin())).ln() / PI;

    if x < 0.0 {
        x = 0.0;
    }
    if x > 1.0 {
        x = 1.0;
    }
    if y < 0.0 {
        y = 0.0;
    }
    if y > 1.0 {
        y = 1.0;
    }

    (x, y)
}

fn project_mercator_to_lonlat(xy: (f64, f64)) -> (i32, i32) {
    let x = xy.0;
    let y = xy.1;

    let lon = x * 360.0 - 180.0;
    let lat = f64::atan(f64::sinh(PI * (1.0 - 2.0 * y))) * 180.0 / PI;

    let lon = (lon * 10_000_000f64) as i32;
    let lat = (lat * 10_000_000f64) as i32;

    (lon, lat)
}


pub fn lonlat_to_xy(lonlat: (i32, i32)) -> (u32, u32) {
    let floats = project_lonlat_to_mercator(lonlat);
    
    let x = (floats.0 as f64 * (u32::MAX as f64)) as u32;
    let y = (floats.1 as f64 * (u32::MAX as f64)) as u32;
    (x, y)
}

pub fn xy_to_lonlat(xy: (u32, u32)) -> (i32, i32) {
    let x = xy.0 as f64 / (u32::MAX as f64);
    let y = xy.1 as f64 / (u32::MAX as f64);

    project_mercator_to_lonlat((x, y))
}

/// Only use this for debugging. There is precision loss. Stay in DM7 when possible.
pub fn lonlat_to_decimal_lonlat(lonlat: (i32, i32)) -> (f64, f64) {
    let dlon = lonlat.0 as f64 / 10_000_000f64;
    let dlat = lonlat.1 as f64 / 10_000_000f64;
    (dlon, dlat)
}

/// Only use this for debugging. There is precision loss. Stay in DM7 when possible.
pub fn decimal_lonlat_to_lonlat(lonlat: (f64, f64)) -> (i32, i32) {
    let lon = (lonlat.0 * 10_000_000f64) as i32;
    let lat = (lonlat.1 * 10_000_000f64) as i32;
    (lon, lat)
}

pub fn xy_to_decimal_lonlat(xy: (u32, u32)) -> (f64, f64) {
    lonlat_to_decimal_lonlat(xy_to_lonlat(xy))
}

pub fn lonlat_to_h(lonlat: (i32, i32)) -> u64 {
    let (x, y) = lonlat_to_xy(lonlat);
    xy2h(x, y, 32)
}

pub fn decimal_lonlat_to_h(dec_lonlat: (f64, f64)) -> u64 {
    let lonlat = decimal_lonlat_to_lonlat(dec_lonlat);
    lonlat_to_h(lonlat)
}

pub fn h_to_xy(h: u64) -> (u32, u32) {
    h2xy(h, 32)
}

pub fn h_to_lonlat(h: u64) -> (i32, i32) {
    let xy = h2xy(h, 32);
    xy_to_lonlat(xy)
}

pub fn h_to_decimal_lonlat(h: u64) -> (f64, f64) {
    let xy = h2xy(h, 32);
    let lonlat = xy_to_lonlat(xy);
    lonlat_to_decimal_lonlat(lonlat)
}

pub fn h_to_zoom_h(h: u64, z: u8) -> u64 {
    h >> (2 * (32 - z))
}

pub fn zoom_h_to_h(h: u64, z: u8) -> u64 {
    h << (2 * (32 - z))
}

// Tile Extent

pub fn extent_for_zoom(z: u8) -> u32 {
    u32::MAX >> z
}

pub fn h_range_for_zoom(z: u8) -> u64 {
    u64::MAX >> z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::*;

    #[test]
    pub fn test_project_lonlat_to_mercator() {
        let lonlat = (0, 0);
        let mercator = project_lonlat_to_mercator(lonlat);
        assert_eq!(mercator, (0.5, 0.5));
    }

    #[test]
    pub fn test_lonlat_to_xy() {
        let lonlat = (0, 0);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (i32::MAX as u32, i32::MAX as u32));
    }

    #[test]
    pub fn test_xy_to_lonlat() {
        let xy = (i32::MAX as u32, i32::MAX as u32);
        let lonlat = xy_to_lonlat(xy);
        assert_eq!(lonlat, (0, 0));
    }

    #[test]
    pub fn test_round_robin() {
        // Cavallero Transit Center
        // -122.0279745, 37.0491457,
        let dec_lonlat = (-122.0279745, 37.0491457);
        let lonlat = decimal_lonlat_to_lonlat(dec_lonlat);
        assert_eq!(lonlat, (-1220279745, 370491457));

        // let xy = lonlat_to_xy(lonlat);
        // assert_eq!(xy, (0x7FFFFFFF, 0x7FFFFFFF));

    }

}
