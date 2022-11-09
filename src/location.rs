#![allow(dead_code)]

use fast_hilbert::{h2xy, xy2h};
use std::f64::consts::PI;

// i32 can contain 1 less value than u32 due to the fact that the MSB needs to store the sign.
const I32_SIZE: f64 = u32::MAX as f64;
const U32_SIZE: f64 = u32::MAX as f64 + 1.0;

/// Projects dm7 lonlat to Web Mercator points in float space (0.0 -> 1.0).
fn project_lonlat_to_mercator(lonlat: (i32, i32)) -> (f64, f64) {
    let lon = lonlat.0 as f64 / 10_000_000f64;
    let lat = lonlat.1 as f64 / 10_000_000f64;
    let mut x = lon / 360.0 + 0.5;

    let mut y = lat * PI / 180.0;
    y = (1.0 - (y.tan() + (1.0 / y.cos())).ln() / PI) / 2.0;

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
    let y = 180.0 - xy.1 * 360.0; // Flip y axis

    let lon = x * 360.0 - 180.0;
    let lat = 360.0 / PI * (y * PI / 180.0).exp().atan() - 90.0;

    let lon = (lon * 10_000_000f64) as i32;
    let lat = (lat * 10_000_000f64) as i32;

    (lon, lat)
}

pub fn lonlat_to_xy(lonlat: (i32, i32)) -> (u32, u32) {
    let floats = project_lonlat_to_mercator(lonlat);

    let x = (floats.0 as f64 * U32_SIZE) as u32;
    let y = (floats.1 as f64 * U32_SIZE) as u32;
    (x, y)
}

pub fn xy_to_lonlat(xy: (u32, u32)) -> (i32, i32) {
    let x = xy.0 as f64 / I32_SIZE;
    let y = xy.1 as f64 / I32_SIZE;

    project_mercator_to_lonlat((x, y))
}

/// There is precision loss here. Stay in DM7 when possible.
pub fn lonlat_to_decimal_lonlat(lonlat: (i32, i32)) -> (f64, f64) {
    let dlon = lonlat.0 as f64 / 10_000_000f64;
    let dlat = lonlat.1 as f64 / 10_000_000f64;
    (dlon, dlat)
}

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
    pub fn test_project_is_in_tile() {
        // Cavallero Transit Center
        // -122.0279745, 37.0491457,
        let lonlat = (-1220279745, 370491457);
        let merc = project_lonlat_to_mercator(lonlat);
        println!("merc: {:?}", merc);

        let t = Tile::from_zxy(22, 675423, 1631832);
        let tile_x = (merc.0 * t.axis_tile_count()) as u32;
        let tile_y = (merc.1 * t.axis_tile_count()) as u32;
        assert_eq!(tile_x, 675423);
        assert_eq!(tile_y, 1631832);

        assert!(merc.0 > 0.16103339195251465 && merc.0 < 0.16103363037109375);
        assert!(merc.1 > 0.38905906677246094 && merc.1 < 0.38905930519104004);
    }

    #[test]
    pub fn test_project_mercator_to_lonlat_cavallero() {
        // Cavallero Transit Center
        // -122.0279745, 37.0491457,
        let lonlat = (-1220279745, 370491457);
        let merc = project_lonlat_to_mercator(lonlat);
        let lonlat2 = project_mercator_to_lonlat(merc);
        assert_eq!(lonlat.0, lonlat2.0);
        // slight loss in precision
        assert_eq!(lonlat.1 / 10, lonlat2.1 / 10);
    }

    #[test]
    pub fn test_project_mercator_to_lonlat() {
        let merc = (0.5, 0.5);
        let lonlat = project_mercator_to_lonlat(merc);
        assert_eq!(lonlat, (0, 0));

        let merc = (0.0, 0.0);
        let lonlat = project_mercator_to_lonlat(merc);
        // Remember, y axis is flipped.
        // Also, this number is correct.
        // See https://en.wikipedia.org/wiki/Web_Mercator_projection
        assert_eq!(lonlat, (-1800000000, 850511287));

        let merc = (1.0, 1.0);
        let lonlat = project_mercator_to_lonlat(merc);
        assert_eq!(lonlat, (1800000000, -850511287));
    }

    #[test]
    pub fn test_lonlat_to_xy() {
        let middle = i32::MAX as u32 + 1;

        let lonlat = (0, 0);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (middle, middle));

        // Cavallero Transit Center
        let lonlat = (-1220279745, 370491457);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (691633204, 1670996018));

        // Origin
        let lonlat = (-1800000000, 900000000);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (0, 0));
    }

    #[test]
    pub fn test_xy_to_lonlat() {
        let xy = (i32::MAX as u32, i32::MAX as u32);
        let lonlat = xy_to_lonlat(xy);
        assert_eq!(lonlat, (0, 0));

        // Cavallero Transit Center
        let xy = (691633204, 1670996018);
        let lonlat = xy_to_lonlat(xy);
        assert_eq!(lonlat, (-1220279745, 370491457));

        // Origin
        let xy = (0, 0);
        let lonlat = xy_to_lonlat(xy);
        assert_eq!(lonlat, (-1800000000, 850511287));
    }

    #[test]
    pub fn test_lonlat_to_xy_and_back_again() {
        // Origin
        let xy = (0, 0);
        let lonlat = xy_to_lonlat(xy);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (0, 11));

        // End of the world
        let lonlat = (1800000000, -900000000);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (u32::MAX, u32::MAX));

        let lonlat = xy_to_lonlat(xy);
        assert_eq!(lonlat, (1800000000, -850511287));
    }

    #[test]
    pub fn test_lonlat_to_h() {
        let lonlat = (-1800000000, 900000000);
        let h = lonlat_to_h(lonlat);
        assert_eq!(h, 0);

        // Bering Strait
        let lonlat = (1800000000, 900000000);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (u32::MAX, 0));

        let h = xy2h(xy.0, xy.1, 32);
        assert_eq!(h, u64::MAX);

        let h = lonlat_to_h(lonlat);
        assert_eq!(h, u64::MAX);
    }

    #[test]
    pub fn test_lonlat_to_h_null_island() {
        let middle = i32::MAX as u32 + 1;

        let lonlat = (0, 0);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (middle, middle));

        // Null Island Tile
        let t = Tile::from_zxy(1, 1, 1).at_zoom(32);
        assert_eq!(t.x, i32::MAX as u32 + 1);
        assert_eq!(t.y, i32::MAX as u32 + 1);
        let null_island_h = t.h;

        let h = lonlat_to_h(lonlat);
        assert_eq!(h, null_island_h);
    }

    #[test]
    pub fn test_lonlat_to_h_cavallero() {
        // Cavallero Transit Center
        let lonlat = (-1220279745, 370491457);
        let xy = lonlat_to_xy(lonlat);
        assert_eq!(xy, (691633204, 1670996018));

        let h = lonlat_to_h(lonlat);
        assert_eq!(h, 3660422102463285814);

        let h2 = xy2h(xy.0, xy.1, 32);
        assert_eq!(h, h2);
    }
}
