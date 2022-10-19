use fast_hilbert::{h2xy, xy2h};
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
// 2^9 = 524,288 , so zoom 9 and above does not quantize coordinates
// Zooms 8 and below do quantize coordinates.

const TILE_EXTENT: u64 = 8192;

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

    // We love Antarctica, but the id may modulus over at a very high zoom.
    pub fn id(&self) -> u64 {
        // z gets 5 bits, max 31
        // h gets 59 bits, max 576460752303423487
        let z = self.z as u64;
        let h = if self.h > 576460752303423487 { self.h % 576460752303423487 } else { self.h };
        z << 59 | h
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
        let children = Vec::from(self.children());
        let mut desc = children.clone();
        let mut q = children;
        for t in q.pop() {
            let children = t.children();
            desc.append(&mut Vec::from(children));
            if t.z <= self.z + child_levels {
                q.append(&mut Vec::from(children));
            }
        }
        desc
    }

    pub fn tree(&self, child_levels: u8) -> Vec<Tile> {
        if self.z == 0 {
            return Vec::<Tile>::new();
        }

        // Get parents
        let mut t = self.clone();
        let mut tree = vec![t];
        for _ in self.z..1 {
            tree.push(t);
            t = t.parent();
        }
        tree.reverse();
        tree.append(&mut self.descendants(child_levels));
        tree
    }

    pub fn bbox(&self) -> BBox {
        let origin = self.origin_location();
        let extent = self.location_extent();
        BBox {
            nw: origin,
            se: PVTPoint::new(origin.x() + extent, origin.y() + extent)
        }
    }

    // Projects a point from location space to tile space.
    pub fn project(&self, point: PVTPoint) -> PVTPoint {

        // PVTPoint shouldnt be u32. Just make it a f64...
        let loc_x = point.x() as u64;
        let loc_y = point.y() as u64;
        let u32_max = u32::MAX as u64;

        let mut x = (loc_x * TILE_EXTENT) / u32_max;
        let mut y = (loc_y * TILE_EXTENT) /u32_max;

        // origin of the tile in the tile's resolution
        let origin_x = self.x as u64 * TILE_EXTENT;
        let origin_y = self.y as u64 * TILE_EXTENT;

        // world origin to tile origin
        x = x - origin_x;
        y = y - origin_y;

        // We shouldn't be clamping exactly to the bounds. 
        // Need to change fb to be f64...

        // clamp to origin
        if x < origin_x {
            x = origin_x
        } else {
            x = x - origin_x;
            // clamp to extent
            if x > TILE_EXTENT {
                x = TILE_EXTENT
            }
        }

        // clamp to origin
        if y < origin_y {
            y = origin_y
        } else {
            y = y - origin_y;
            // clamp to extent
            if y > TILE_EXTENT {
                y = TILE_EXTENT
            }
        }

        PVTPoint::new(x as u32, y as u32)
    }

    pub fn project_bbox(&self, bbox: BBox) -> BBox {
        BBox {
            nw: self.project(bbox.nw),
            se: self.project(bbox.se)
        }
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
    se: PVTPoint
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

// let tile = await planet.tile(9, 82, 199);
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
    fn shift() {
        let max: u32 = 4294967295;
        let half: u32 = 2147483647;
        assert_eq!(max >> 1, half);
    }
}