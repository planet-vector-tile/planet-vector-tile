#![allow(dead_code)]

use fast_hilbert::{h2xy, xy2h};
use queue::Queue;
use std::{fmt, ops::Range};

#[allow(dead_code, unused_imports)]
#[path = "./generated/planet_vector_tile_generated.rs"]
pub mod planet_vector_tile_generated;
use planet_vector_tile_generated::*;

// Look into using simd...
// use std::simd::u32x2;

// The max u32 is 4,294,967,295 (2^32),
// so the unit location tile would be zoom 32.

// In MapboxGL, the max extent for a tile is 8192 (13 bit unsigned integer)
// 4,294,967,296 / 8192 = 524,288
// 2^9 = 524,288 , so zoom 9 and above does not quantize coordinates.
// Zooms 8 and below do quantize coordinates.

const U32_SIZE: f64 = u32::MAX as f64 + 1_f64;

// https://github.com/maplibre/maplibre-gl-js/blob/9aabd047281ac94c246a8ebedb850ff1133a0407/src/data/extent.ts#L16
const TILE_EXTENT: f64 = 8192_f64;

// https://github.com/maplibre/maplibre-gl-js/blob/9aabd047281ac94c246a8ebedb850ff1133a0407/src/data/load_geometry.ts#L12-L14
const TILE_MAX: f64 = 16383_f64;
const TILE_MIN: f64 = -16384_f64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tile {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub h: u64,
}

impl Tile {
    pub fn default() -> Self {
        Self {
            z: 0,
            x: 0,
            y: 0,
            h: 0,
        }
    }

    pub fn from_zh(z: u8, h: u64) -> Self {
        if z == 0 {
            return Self {
                z: 0,
                x: 0,
                y: 0,
                h: 0,
            };
        }
        let (x, y) = h2xy(h, z);
        Self { z, x, y, h }
    }

    pub fn from_zxy(z: u8, x: u32, y: u32) -> Self {
        if z == 0 {
            return Self {
                z: 0,
                x: 0,
                y: 0,
                h: 0,
            };
        }
        let h = xy2h(x, y, z);
        Self { z, x, y, h }
    }

    pub fn at_zoom(&self, z: u8) -> Tile {
        if z == self.z {
            self.clone()
        } else if z == 0 {
            Tile {
                z: 0,
                x: 0,
                y: 0,
                h: 0,
            }
        } else if z == 32 && self.z == 0 {
            Tile::from_zxy(32, 0, 0)
        } else if z > self.z {
            let z_delta = (z - self.z) as u32;
            let x = self.x << z_delta;
            let y = self.y << z_delta;
            Tile::from_zxy(z, x, y)
        } else {
            let z_delta = (self.z - z) as u32;
            let x = self.x >> z_delta;
            let y = self.y >> z_delta;
            Tile::from_zxy(z, x, y)
        }
    }

    pub fn h_range_for_zoom(&self, z: u8) -> Range<u64> {
        // The range for this tile
        if z == self.z {
            Range {
                start: self.h,
                end: self.h + 1,
            }
        }
        // The range for a lower zoom is the shifted h
        // with the length of 1
        else if z < self.z {
            let start = self.h >> 2 * (self.z - z) as u64;
            Range {
                start,
                end: start + 1,
            }
        }
        // For a higher zoom, we shift h to the corresponding Hilbert order,
        // then we determine the number of tiles this tile covers
        // to compute the range.
        else {
            let z_delta = (z - self.z) as u64;
            let start = self.h << 2 * z_delta;
            Range {
                start,
                end: start + (1 << 2 * z_delta),
            }
        }
    }

    // The origin is the Northwest corner.
    // This is the coordinate of the tile in location space (zoom 32).
    pub fn location_tile(&self) -> Tile {
        self.at_zoom(32)
    }

    // The id has a max of 52 bits to accomodate JavaScript numbers.
    // z gets 5 bits, max 31
    // h gets 47 bits, max 140_737_488_355_327
    // https://www.rustexplorer.com/b/0uno7c
    pub fn id(&self) -> u64 {
        let z = self.z as u64;

        // 2^47 - 1 , this limit is within z24.
        if self.h > 140_737_488_355_327 {
            // When we go above the limit, we just omit the z from the id.
            // 2 ^ 52
            return self.h % 4_503_599_627_370_495;
        }

        z << 47 | self.h
    }

    // The Northwest corner of the tile in location space.
    pub fn origin_location(&self) -> (u32, u32) {
        if self.z == 0 {
            (0, 0)
        } else if self.z == 32 {
            (self.x, self.y)
        } else {
            let z_delta = 32 - self.z as u32;
            let x = self.x << z_delta;
            let y = self.y << z_delta;
            (x, y)
        }
    }

    pub fn origin_float(&self) -> (f64, f64) {
        let count = self.axis_tile_count();
        let x = self.x as f64 / count as f64;
        let y = self.y as f64 / count as f64;
        (x, y)
    }

    // The extent of a tile in location space
    pub fn location_extent(&self) -> u32 {
        if self.z == 32 {
            return 0;
        }
        u32::MAX >> self.z
    }

    // The center in location space
    pub fn center(&self) -> (u32, u32) {
        let middle = self.location_extent() >> 1;
        let origin = self.origin_location();
        (origin.0 + middle, origin.1 + middle)
    }

    pub fn parent(&self) -> Option<Self> {
        if self.z == 0 {
            None
        } else {
            Some(Self {
                z: self.z - 1,
                x: self.x >> 1,
                y: self.y >> 1,
                h: self.h >> 2,
            })
        }
    }

    fn ancestor(&self, z: u8) -> Self {
        assert!(z < self.z);
        if z == 0 {
            return Self {
                z: 0,
                x: 0,
                y: 0,
                h: 0,
            };
        }
        Self {
            z,
            x: self.x >> self.z - z,
            y: self.y >> self.z - z,
            h: self.h >> (self.z - z) * 2,
        }
    }

    pub fn children(&self) -> [Tile; 4] {
        let z = self.z + 1;
        let w = self.x * 2;
        let n = self.y * 2;
        let nw = Tile::from_zxy(z, w, n);
        let sw = Tile::from_zxy(z, w, n + 1);
        let se = Tile::from_zxy(z, w + 1, n + 1);
        let ne = Tile::from_zxy(z, w + 1, n);
        [nw, sw, se, ne]
    }

    pub fn grandchildren(&self) -> [Tile; 16] {
        let mut tiles = [Tile::default(); 16];
        let mut i = 0;
        for child in self.children().iter() {
            for grand_child in child.children().iter() {
                tiles[i] = *grand_child;
                i += 1;
            }
        }
        tiles
    }

    pub fn descendants(&self, child_levels: u8) -> Vec<Tile> {
        if child_levels == 0 {
            return Vec::<Tile>::new();
        }
        let top_z = if self.z + child_levels >= 28 {
            28
        } else {
            self.z + child_levels
        };
        let mut desc = Vec::<Tile>::new();
        let mut q = Queue::<Tile>::new();
        for t in self.grandchildren() {
            q.queue(t).unwrap();
        }
        while !q.is_empty() {
            let t = q.dequeue().unwrap();
            if t.z <= top_z {
                for c in t.grandchildren() {
                    q.queue(c).unwrap();
                }
            }
            desc.push(t);
        }

        desc
    }

    pub fn pyramid(&self, child_levels: u8) -> Vec<Tile> {
        let size = self.z as usize + 1 + (1 << 2 * child_levels);

        let mut pyramid = Vec::<Tile>::with_capacity(size);

        for z in (0..self.z).step_by(2) {
            pyramid.push(self.ancestor(z));
        }
        pyramid.push(self.clone());
        pyramid.append(&mut self.descendants(child_levels));
        pyramid
    }

    pub fn bbox(&self) -> BBox {
        let origin = self.origin_location();
        let extent = self.location_extent();
        BBox {
            nw: origin,
            se: (origin.0 + extent, origin.1 + extent),
        }
    }

    pub fn axis_tile_count(&self) -> f64 {
        axis_tile_count(self.z)
    }

    // Projects a point from location space to tile space.
    pub fn project(&self, loc: (u32, u32)) -> PVTTilePoint {
        // location in planet resolution
        let loc_x = loc.0 as f64;
        let loc_y = loc.1 as f64;

        // where coord is between 0 -> 1 for planet space
        let unit_x = loc_x / U32_SIZE;
        let unit_y = loc_y / U32_SIZE;

        let resolution = self.axis_tile_count() * TILE_EXTENT;
        let tile_x = unit_x * resolution;
        let tile_y = unit_y * resolution;

        let origin_x = self.x as f64 * TILE_EXTENT;
        let origin_y = self.y as f64 * TILE_EXTENT;

        let mut x = tile_x - origin_x;
        let mut y = tile_y - origin_y;

        // TODO: Provide offset around the tile bounds for clamping.

        // clamp
        if x < TILE_MIN {
            x = TILE_MIN;
        }
        if x > TILE_MAX {
            x = TILE_MAX;
        }
        if y < TILE_MIN {
            y = TILE_MIN;
        }
        if y > TILE_MAX {
            y = TILE_MAX;
        }

        PVTTilePoint::new(x as i16, y as i16)
    }

    pub fn hilbert_bearing(&self) -> HilbertBearing {
        let hilbert_order_max: u32 = 1_u32 << self.z;

        let n = if self.y != 0 {
            Some(xy2h(self.x, self.y - 1, self.z))
        } else {
            None
        };

        // w
        let w = if self.x != 0 {
            Some(xy2h(self.x - 1, self.y, self.z))
        } else {
            None
        };

        // s
        let s = if self.y + 1 < hilbert_order_max {
            Some(xy2h(self.x, self.y + 1, self.z))
        } else {
            None
        };

        // e
        let e = if self.x + 1 < hilbert_order_max {
            Some(xy2h(self.x + 1, self.y, self.z))
        } else {
            None
        };

        let from_h = if self.h != 0 { Some(self.h - 1) } else { None };
        let to_h = if self.h < (1 << (2 * self.z)) {
            Some(self.h + 1)
        } else {
            None
        };

        if to_h.is_none() {
            return HilbertBearing::None;
        }

        if n == from_h {
            if w == to_h {
                HilbertBearing::NW
            } else if s == to_h {
                HilbertBearing::NS
            } else if e == to_h {
                HilbertBearing::NE
            } else {
                HilbertBearing::None
            }
        } else if w == from_h {
            if s == to_h {
                HilbertBearing::WS
            } else if e == to_h {
                HilbertBearing::WE
            } else if n == to_h {
                HilbertBearing::WN
            } else {
                HilbertBearing::WE
            }
        } else if s == from_h {
            if e == to_h {
                HilbertBearing::SE
            } else if n == to_h {
                HilbertBearing::SN
            } else if w == to_h {
                HilbertBearing::SW
            } else {
                HilbertBearing::SN
            }
        } else if e == from_h {
            if n == to_h {
                HilbertBearing::EN
            } else if w == to_h {
                HilbertBearing::EW
            } else if s == to_h {
                HilbertBearing::ES
            } else {
                HilbertBearing::None
            }
        } else {
            HilbertBearing::None
        }
    }
}

pub fn axis_tile_count(z: u8) -> f64 {
    if z == 0 {
        1_f64
    } else if z == 32 {
        U32_SIZE
    } else {
        (2u32 << (z - 1) as u32) as f64
    }
}

pub fn tile_count_for_zoom(z: u8) -> u128 {
    if z == 0 {
        1_u128
    } else if z == 32 {
        u64::MAX as u128 + 1
    } else {
        1_u128 << (2 * z)
    }
}

impl Eq for Tile {}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "z{} x{} y{} h{}", self.z, self.x, self.y, self.h)
    }
}

pub struct BBox {
    // min
    nw: (u32, u32),
    // max
    se: (u32, u32),
}

impl BBox {
    pub fn nw(&self) -> (u32, u32) {
        self.nw
    }
    pub fn sw(&self) -> (u32, u32) {
        (self.nw.0, self.se.1)
    }
    pub fn se(&self) -> (u32, u32) {
        self.se
    }
    pub fn ne(&self) -> (u32, u32) {
        (self.se.0, self.nw.1)
    }
}

pub enum HilbertBearing {
    NW,
    NS,
    NE,
    WS,
    WE,
    WN,
    SE,
    SN,
    SW,
    EN,
    EW,
    ES,
    None,
}

#[cfg(test)]
mod tests {
    // use crate::location::lonlat_to_xy;

    use super::*;

    #[test]
    fn test_basic_tile() {
        let t = Tile::from_zxy(9, 82, 199);
        // let t2 = Tile::from_zxy(9, 83, 300);

        // assert_eq!(t.id(), 5188146770730836249);
        // assert_eq!(t2.id(), 5188146770731048437);

        let t_loc_tile = t.location_tile();
        assert_eq!(t_loc_tile.z, 32);
        assert_eq!(t_loc_tile.x, 687865856);
        assert_eq!(t_loc_tile.y, 1669332992);
        assert_eq!(t_loc_tile.h, 3660417878385666730);
    }

    #[test]
    fn test_at_zoom() {
        let t = Tile::from_zxy(0, 0, 0);
        let zt = t.at_zoom(1);
        assert_eq!(zt.z, 1);
        assert_eq!(zt.x, 0);
        assert_eq!(zt.y, 0);
        assert_eq!(zt.h, 0);

        let zt32 = t.at_zoom(2);
        assert_eq!(zt32.z, 2);
        assert_eq!(zt32.x, 0);
        assert_eq!(zt32.y, 0);
        assert_eq!(zt32.h, 0);

        let zt31 = t.at_zoom(31);
        assert_eq!(zt31.z, 31);
        assert_eq!(zt31.x, 0);
        assert_eq!(zt31.y, 0);
        assert_eq!(zt31.h, 0);

        let zt32 = t.at_zoom(32);
        assert_eq!(zt32.z, 32);
        assert_eq!(zt32.x, 0);
        assert_eq!(zt32.y, 0);
        assert_eq!(zt32.h, 0);

        let barrow = Tile::from_zh(5, 24);

        assert_eq!(barrow.at_zoom(4).h, 6);
        assert_eq!(barrow.at_zoom(3).h, 1);
        assert_eq!(barrow.at_zoom(2).h, 0);
        assert_eq!(barrow.at_zoom(1).h, 0);
        assert_eq!(barrow.at_zoom(0).h, 0);

        assert_eq!(barrow.at_zoom(6).h, 96);
        assert_eq!(barrow.at_zoom(7).h, 384);
        assert_eq!(barrow.at_zoom(8).h, 1536);
        assert_eq!(barrow.at_zoom(9).h, 6144);
        assert_eq!(barrow.at_zoom(10).h, 24576);

        // "z": 28,
        // "x": 43227083,
        // "y": 104437248,
        // "h": 14298523837747280
        let cavallero = Tile::from_zh(28, 14298523837747280);

        assert_eq!(cavallero.at_zoom(4).h, 50);
        assert_eq!(cavallero.at_zoom(3).h, 12);
        assert_eq!(cavallero.at_zoom(2).h, 3);
        assert_eq!(cavallero.at_zoom(1).h, 0);
        assert_eq!(cavallero.at_zoom(0).h, 0);

        assert_eq!(cavallero.at_zoom(8).h, 13004);
        assert_eq!(cavallero.at_zoom(9).h, 52017);
        assert_eq!(cavallero.at_zoom(10).h, 208070);
    }

    #[test]
    fn test_location_extent() {
        let t = Tile::from_zxy(0, 0, 0);
        let e = t.location_extent();
        assert_eq!(e, 4294967295);

        let t = Tile::from_zxy(1, 0, 0);
        let e = t.location_extent();
        assert_eq!(e, 2147483647);
    }

    #[test]
    fn test_bbox() {
        let t = Tile::from_zxy(0, 0, 0);
        let b = t.bbox();
        assert_eq!(b.nw.0, 0);
        assert_eq!(b.nw.1, 0);
        assert_eq!(b.se.0, 4294967295);
        assert_eq!(b.se.1, 4294967295);

        let b2 = Tile::from_zxy(1, 0, 0).bbox();
        assert_eq!(b2.nw.0, 0);
        assert_eq!(b2.nw.1, 0);
        assert_eq!(b2.se.0, 2147483647);
        assert_eq!(b2.se.1, 2147483647);

        let b3 = Tile::from_zxy(1, 1, 0).bbox();
        assert_eq!(b3.nw.0, 2147483648);
        assert_eq!(b3.nw.1, 0);
        assert_eq!(b3.se.0, 4294967295);
        assert_eq!(b3.se.1, 2147483647);
    }

    #[test]
    fn test_center() {
        let c = Tile::from_zxy(32, 0, 0).center();
        assert_eq!(c.0, 0);
        assert_eq!(c.1, 0);

        let c2 = Tile::from_zxy(31, 0, 0).center();
        assert_eq!(c2.0, 0);
        assert_eq!(c2.1, 0);

        let c3 = Tile::from_zxy(0, 0, 0).center();
        assert_eq!(c3.0, 2147483647);
        assert_eq!(c3.1, 2147483647);

        // Is this right?
        let c4 = Tile::from_zxy(30, 0, 0).center();
        assert_eq!(c4.0, 1);
        assert_eq!(c4.1, 1);
    }

    #[test]
    fn test_pyramid() {
        let t = Tile::from_zxy(0, 0, 0);
        let p = t.pyramid(0);
        assert_eq!(p.len(), 1);
        let p2 = t.pyramid(1);
        assert_eq!(p2.len(), 5);
    }

    #[test]
    fn test_h_range_for_zoom() {
        let t = Tile::from_zxy(9, 82, 199);
        // let t = Tile::from_zh(9, 52017);
        let range = t.h_range_for_zoom(9);
        assert_eq!(range.start, 52017);
        assert_eq!(range.end, 52018);

        let range2 = t.h_range_for_zoom(2);
        assert_eq!(range2.start, 3);
        assert_eq!(range2.end, 4);

        let range3 = t.h_range_for_zoom(12);
        assert_eq!(range3.start, 3_329_088);
        assert_eq!(range3.end, 3_329_152);
    }

    #[test]
    fn test_axis_tile_count() {
        assert_eq!(axis_tile_count(0), 1_f64);
        assert_eq!(axis_tile_count(1), 2_f64);
        assert_eq!(axis_tile_count(2), 4_f64);
        assert_eq!(axis_tile_count(3), 8_f64);
        assert_eq!(axis_tile_count(31), 2147483648_f64);
        assert_eq!(axis_tile_count(32), 4294967296_f64);
    }

    #[test]
    fn test_tile_count_for_zoom() {
        assert_eq!(tile_count_for_zoom(0), 1_u128);
        assert_eq!(tile_count_for_zoom(1), 4_u128);
        assert_eq!(tile_count_for_zoom(2), 16_u128);
        assert_eq!(tile_count_for_zoom(3), 64_u128);
        assert_eq!(tile_count_for_zoom(31), 4611686018427387904_u128);
        assert_eq!(tile_count_for_zoom(32), 18446744073709551616_u128);
    }
}
