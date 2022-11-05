// LonLat, XY (unsigned LonLat), Hilbert Location conversion

use std::f64::consts::PI;

use fast_hilbert::{xy2h, h2xy};

/// Projects dm7 lonlat to mercator points in float space (0.0 -> 1.0)
pub fn project_lonlat_to_mercator(lonlat: (i32, i32)) -> (f64, f64) {
    let mut lon = lonlat.0 as f64 / 10_000_000f64;
    let mut lat = lonlat.1 as f64 / 10_000_000f64;

    // Make x be 0..1
    let mut x: f64 = (lon + 180.0) / 360.0;

    let sin: f64 = f64::sin(lat * PI / 180.0);
    let mut y = 0.5 - 0.25 * f64::ln((1.0 + sin) / (1.0 - sin) / PI);

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


pub fn lonlat_to_xy(lonlat: (i32, i32)) -> (u32, u32) {
    let floats = project_lonlat_to_mercator(lonlat);
    
    let x = (floats.0 as f64 * (u32::MAX as f64)) as u32;
    let y = (floats.1 as f64 * (u32::MAX as f64)) as u32;
    (x, y)
}

pub fn xy_to_lonlat(xy: (u32, u32)) -> (i32, i32) {
    let x = xy.0 as f64 / (u32::MAX as f64);
    let y = xy.1 as f64 / (u32::MAX as f64);

    let lon = x * 360.0 - 180.0;
    let lat = f64::atan(f64::sinh(PI * (1.0 - 2.0 * y))) * 180.0 / PI;

    let lon = (lon * 10_000_000f64) as i32;
    let lat = (lat * 10_000_000f64) as i32;

    (lon, lat)
}

#[inline(always)]
pub fn lonlat_to_decimal(lonlat: (i32, i32)) -> (f64, f64) {
    let dlon = lonlat.0 as f64 / 10_000_000f64;
    let dlat = lonlat.1 as f64 / 10_000_000f64;
    (dlon, dlat)
}

#[inline(always)]
pub fn xy_to_decimal(xy: (u32, u32)) -> (f64, f64) {
    lonlat_to_decimal(xy_to_lonlat(xy))
}

#[inline(always)]
pub fn lonlat_to_h(lonlat: (i32, i32)) -> u64 {
    let (x, y) = lonlat_to_xy(lonlat);
    xy2h(x, y, 32)
}

#[inline(always)]
pub fn h_to_xy(h: u64) -> (u32, u32) {
    h2xy(h, 32)
}

#[inline(always)]
pub fn h_to_lonlat(h: u64) -> (i32, i32) {
    let xy = h2xy(h, 32);
    xy_to_lonlat(xy)
}

#[inline(always)]
pub fn dm7_h_to_zoom_h(h: u64, z: u8) -> u64 {
    h >> (2 * (32 - z))
}

#[inline(always)]
pub fn zoom_h_to_dm7_h(h: u64, z: u8) -> u64 {
    h << (2 * (32 - z))
}

// Tile Extent

#[inline(always)]
pub fn extent_for_zoom(z: u8) -> u32 {
    u32::MAX >> z
}

#[inline(always)]
pub fn h_range_for_zoom(z: u8) -> u64 {
    u64::MAX >> z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::*;

    #[test]
    pub fn test_lonlat_to_xy() {
        // assert_eq!(lonlat_to_xy((-1213696037, 386494418)), (933787610, 2533978065));
        // assert_eq!(xy_to_lonlat((933787610, 2533978065)), (-1213696037, 386494418));

        println!("{}", i32::MIN);
        println!("{}", i32::MAX);
        println!("{}", u32::MIN);
        println!("{}", u32::MAX);
        // -2147483648
        // 2147483647
        // 0
        // 4294967295
        
        assert_eq!(lonlat_to_xy((i32::MAX, i32::MAX)), (u32::MAX, u32::MAX));
        // assert_eq!(lonlat_to_xy((0, 0)), (i32::MIN, i32::MIN));
    }

    #[test]
    pub fn test_xy_to_lonlat() {
        // let lon = ((xy.0 as i64) - (i32::MAX as i64)) as i32;
        // let lat = ((xy.1 as i64) - (i32::MAX as i64)) as i32;

        // let x: u32 = 0;
        // let x1 = x as i64;
        // let max_i32 = i32::MAX as i64;
        // let x2 = x1 - max_i32 - 1;
        // let x3 = x2 as i32;

        // println!("{}", x1);
        // println!("{}", max_i32);
        // println!("{}", x2);
        // println!("{}", x3);

        assert_eq!(xy_to_lonlat((4294967295, 4294967295)), (2147483647, 2147483647));
        assert_eq!(xy_to_lonlat((0, 0)), (-2147483648, -2147483648));
        assert_eq!(xy_to_lonlat((u32::MAX, u32::MAX)), (i32::MAX, i32::MAX));
        assert_eq!(xy_to_lonlat((0, 0)), (i32::MIN, i32::MIN));
    }

    #[test]
    pub fn test_lonlat_to_h() {
        // Cavallero Transit Center
        assert_eq!(lonlat_to_h((-1220279745, 370491457)), 5056332410240376830);

        assert_eq!(lonlat_to_h((-1220267360, 369514859)), 5056328721337122201);
        assert_eq!(lonlat_to_h((-1220267093, 369514589)), 5056328721336989468);
    }

    #[test]
    pub fn test_dm7_h_to_zoom_h() {
        // Cavallero Transit Center
        let dm7_h = 5056332410240376830;
        let z_h = dm7_h_to_zoom_h(dm7_h, 12);
        // assert_eq!(z_h, 3329134);

        // 3329134
        // 4598707

        let huh = Tile::from_zh(12, 4598707);
        println!("{:?}", huh);
        // Tile { z: 12, x: 884, y: 2401, h: 4598707 }

        let tile = Tile::from_zh(12, 3329134);
        assert_eq!(tile.x, 659);
        assert_eq!(tile.y, 1593);
    }
}
