// LonLat, XY (unsigned LonLat), Hilbert Location conversion

use fast_hilbert::{xy2h, h2xy};

#[inline(always)]
pub fn lonlat_to_xy(lonlat: (i32, i32)) -> (u32, u32) {
    let x = (lonlat.0 as i64 + i32::MAX as i64) as u32;
    let y = (lonlat.1 as i64 + i32::MAX as i64) as u32;
    (x, y)
}

#[inline(always)]
pub fn xy_to_lonlat(xy: (u32, u32)) -> (i32, i32) {
    let lon = (xy.0 as i64 - i32::MAX as i64) as i32;
    let lat = (xy.1 as i64 - i32::MAX as i64) as i32;
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

    #[test]
    pub fn test_lonlat_to_xy() {
        assert_eq!(lonlat_to_xy((-1213696037, 386494418)), (933787610, 2533978065))
    }


}
