use fast_hilbert::{h2xy, xy2h};
use queue::Queue;
use std::fmt;

#[allow(dead_code, unused_imports)]
#[path = "./fbs/planet_vector_tile_generated.rs"]
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

// Why do I get these dead code complaints on functions I am using?
#[allow(dead_code)]
impl Tile {
    pub fn from_zh(z: u8, h: u64) -> Self {
        let (x, y) = h2xy(h);
        Self { z, x, y, h }
    }

    pub fn from_zxy(z: u8, x: u32, y: u32) -> Self {
        let h = xy2h(x, y);
        Self { z, x, y, h }
    }

    pub fn at_zoom(&self, z: u8) -> Tile {
        let z_delta = (z - self.z) as u32;
        if z == self.z {
            self.clone()
        } else if z == 32 && self.z == 0 {
            Tile::from_zxy(32, 0, 0)
        } else if z > self.z {
            let x = self.x << z_delta;
            let y = self.y << z_delta;
            Tile::from_zxy(z, x, y)
        } else {
            let x = self.x >> z_delta;
            let y = self.y >> z_delta;
            Tile::from_zxy(z, x, y)
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
    pub fn origin_location(&self) -> PVTPoint {
        if self.z == 0 {
            PVTPoint::new(0, 0)
        } else if self.z == 32 {
            PVTPoint::new(self.x, self.y)
        } else {
            let z_delta = 32 - self.z as u32;
            let x = self.x << z_delta;
            let y = self.y << z_delta;
            PVTPoint::new(x, y)
        }
    }

    // The extent of a tile in location space
    pub fn location_extent(&self) -> u32 {
        if self.z == 32 {
            return 0;
        }
        u32::MAX >> self.z
    }

    // The center in location space
    pub fn center(&self) -> PVTPoint {
        let middle = self.location_extent() >> 1;
        let origin = self.origin_location();
        PVTPoint::new(origin.x() + middle, origin.y() + middle)
    }

    pub fn parent(&self) -> Tile {
        Tile::from_zxy(self.z - 1, self.x >> 1, self.y >> 1)
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

    pub fn descendants(&self, child_levels: u8) -> Vec<Tile> {
        if child_levels == 0 {
            return Vec::<Tile>::new();
        }
        let top_z = if self.z + child_levels >= 31 { 31 } else { self.z + child_levels };
        let mut desc = Vec::<Tile>::new();
        let mut q = Queue::<Tile>::new();
        for t in self.children() {
            q.queue(t).unwrap();
        }
        while !q.is_empty() {
            let t = q.dequeue().unwrap();
            if t.z < top_z {
                for c in t.children() {
                    q.queue(c).unwrap();
                }
            }
            desc.push(t);
        }

        desc
    }

    pub fn pyramid(&self, child_levels: u8) -> Vec<Tile> {
        // Get parents
        let mut pyramid = Vec::<Tile>::with_capacity(
            (self.z + 1) as usize + 4_u32.pow(child_levels as u32) as usize,
        );
        let mut t = self.clone();
        pyramid.push(t);
        for _ in 0..self.z {
            t = t.parent();
            pyramid.push(t);
        }
        pyramid.reverse();
        pyramid.append(&mut self.descendants(child_levels));
        pyramid
    }

    pub fn bbox(&self) -> BBox {
        let origin = self.origin_location();
        let extent = self.location_extent();
        BBox {
            nw: origin,
            se: PVTPoint::new(origin.x() + extent, origin.y() + extent),
        }
    }

    fn axis_tile_count(&self) -> f64 {
        if self.z == 0 {
            1_f64
        } else if self.z == 32 {
            U32_SIZE
        } else {
            (2u32 << (self.z - 1) as u32) as f64
        }
    }

    // Projects a point from location space to tile space.
    pub fn project(&self, loc: PVTPoint) -> PVTTilePoint {
        // location in planet resolution
        let loc_x = loc.x() as f64;
        let loc_y = loc.y() as f64;

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

        // We shouldn't be clamping exactly to the bounds.
        // Need to change fb to be i16...

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
        // n
        let n = if self.y != 0 {
            Some(xy2h(self.x, self.y - 1))
        } else {
            None
        };

        // w
        let w = if self.x != 0 {
            Some(xy2h(self.x - 1, self.y))
        } else {
            None
        };

        // s
        let s = if self.y != u32::MAX {
            Some(xy2h(self.x, self.y + 1))
        } else {
            None
        };

        // e
        let e = if self.x != u32::MAX {
            Some(xy2h(self.x + 1, self.y))
        } else {
            None
        };

        let from_h = if self.h != 0 { Some(self.h - 1) } else { None };
        let to_h = if self.h != u64::MAX { Some(self.h + 1) } else { None };

        if n == from_h {
            if w == to_h {
                return HilbertBearing::NW;
            }
            if s == to_h {
                return HilbertBearing::NS;
            }
            if e == to_h {
                return HilbertBearing::NE;
            }
        }
        if w == from_h {
            if s == to_h {
                return HilbertBearing::WS;
            }
            if e == to_h {
                return HilbertBearing::WE;
            }
            if n == to_h {
                return HilbertBearing::WN;
            }
        }
        if s == from_h {
            if e == to_h {
                return HilbertBearing::SE;
            }
            if n == to_h {
                return HilbertBearing::SN;
            }
            if w == to_h {
                return HilbertBearing::SW;
            }
        }
        if e == from_h {
            if n == to_h {
                return HilbertBearing::EN;
            }
            if w == to_h {
                return HilbertBearing::EW;
            }
            if s == to_h {
                return HilbertBearing::ES;
            }
        }
        // first tile will always be west to east
        HilbertBearing::WE

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
    nw: PVTPoint,
    // max
    se: PVTPoint,
}

// Why do I get these dead code complaints on functions I am using?
#[allow(dead_code)]
impl BBox {
    pub fn nw(&self) -> PVTPoint {
        self.nw
    }
    pub fn sw(&self) -> PVTPoint {
        PVTPoint::new(self.nw.x(), self.se.y())
    }
    pub fn se(&self) -> PVTPoint {
        self.se
    }
    pub fn ne(&self) -> PVTPoint {
        PVTPoint::new(self.se.x(), self.nw.y())
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
    ES
}

mod tests {
    use super::*;

    #[test]
    fn test_basic_tile() {
        let t = Tile::from_zxy(9, 82, 199);
        let t2 = Tile::from_zxy(9, 83, 300);

        assert_eq!(t.id(), 5188146770730836249);
        assert_eq!(t2.id(), 5188146770731048437);

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
        assert_eq!(b.nw.x(), 0);
        assert_eq!(b.nw.y(), 0);
        assert_eq!(b.se.x(), 4294967295);
        assert_eq!(b.se.y(), 4294967295);

        let b2 = Tile::from_zxy(1, 0, 0).bbox();
        assert_eq!(b2.nw.x(), 0);
        assert_eq!(b2.nw.y(), 0);
        assert_eq!(b2.se.x(), 2147483647);
        assert_eq!(b2.se.y(), 2147483647);

        let b3 = Tile::from_zxy(1, 1, 0).bbox();
        assert_eq!(b3.nw.x(), 2147483648);
        assert_eq!(b3.nw.y(), 0);
        assert_eq!(b3.se.x(), 4294967295);
        assert_eq!(b3.se.y(), 2147483647);
    }

    #[test]
    fn test_center() {
        let c = Tile::from_zxy(32, 0, 0).center();
        assert_eq!(c.x(), 0);
        assert_eq!(c.y(), 0);

        let c2 = Tile::from_zxy(31, 0, 0).center();
        assert_eq!(c2.x(), 0);
        assert_eq!(c2.y(), 0);

        let c3 = Tile::from_zxy(0, 0, 0).center();
        assert_eq!(c3.x(), 2147483647);
        assert_eq!(c3.y(), 2147483647);

        // Is this right?
        let c4 = Tile::from_zxy(30, 0, 0).center();
        assert_eq!(c4.x(), 1);
        assert_eq!(c4.y(), 1);
    }

    #[test]
    fn test_pyramid() {
        let t = Tile::from_zxy(0, 0, 0);
        let p = t.pyramid(0);
        assert_eq!(p.len(), 1);
        let p2 = t.pyramid(1);
        assert_eq!(p2.len(), 5);
    }
}
