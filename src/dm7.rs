// LonLat, XY (unsigned LonLat), Hilbert Location conversion

use fast_hilbert::{xy2h, h2xy};

const I32_PLUS_1: i64 =  (i32::MAX as i64) + 1;
const I32_MINUS_1: i64 =  (i32::MAX as i64) - 1;

#[inline(always)]
pub fn lonlat_to_xy(lonlat: (i32, i32)) -> (u32, u32) {
    let x = ((lonlat.0 as i64) + I32_MINUS_1) as u32;
    let y = ((lonlat.1 as i64) + I32_MINUS_1) as u32;
    (x, y)
}

pub fn xy_to_lonlat(xy: (u32, u32)) -> (i32, i32) {
    // let x = xy.0 as i64;
    // let y = xy.1 as i64;
    // println!("x {} I32_MINUS_1 {}", x, I32_PLUS_1);
    // let lon_i64 = x - I32_PLUS_1;
    // let lat_i64 = y - I32_PLUS_1;
    // let lon = lon_i64 as i32;
    // let lat = lat_i64 as i32;

    let lon = ((xy.0 as i64) + I32_PLUS_1) as i32;
    let lat = ((xy.1 as i64) + I32_PLUS_1) as i32;
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
