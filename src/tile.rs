use fast_hilbert::{h2xy, xy2h};

// max u32 is 4,294,967,296 (2^32)


// u32 for x y is a unit location representation at zoom 16
// 4 ^ 16 = 4,294,967,296
// u32 max is 4,294,967,295

// In MapboxGL, the max extent for a tile is 8192 (13 bit integer)
// 4,294,967,296 / 8192 = 524,288

// 4 ^ 10 = 1,048,576       1,048,576 * 8192 = 8,589,934,592    too big
// 4 ^ 9  = 262,144         262,144   * 8192 = 2,147,483,648    too small
// Any unit location at zoom 10 or above is not quantized.


pub struct Tile {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub h: u64,
}

impl Tile {
    pub fn from_zh(z: u8, h: u64) -> Self {
        let (x, y) = h2xy(h);
        Self { z, x, y, h }
    }

    pub fn from_zxy(z: u8, x: u32, y: u32) -> Self {
        let h = xy2h(x, y);
        Self { z, x, y, h }
    }

    // The origin is the Northwest corner.
    // The location is the xy at zoom 16.
    // You can think of the returned tile as the
    // "Location Tile".
    // pub fn origin_location() -> Tile {

    // }

    // pub fn center_location() -> Tile {

    // }

    // pub fn hilbert_at_zoom(z: u8) -> u64 {
    //     54 as u64
    // }

    // pub fn children(&self) -> [Tile] {
    //     let z = self.z + 1;
    //     let w = self.x * 2;
    //     let n = self.y * 2;
    //     let nw = Self::from_zxy(z, w, n);
    //     let sw = Self::from_zxy(z, w, n + 1);
    //     let se = Self::from_zxy(z, w + 1, n + 1);
    //     let ne = Self::from_zxy(z, w + 1, n);
    //     return [nw, sw, se, ne];
    // }


}

pub struct BBox {
    w: u32,
    s: u32,
    e: u32,
    n: u32
}
