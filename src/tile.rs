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

const EXTENT: u32 = 8192;

#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn at_zoom(&self, z: u8) -> Tile {
        let z_delta = (z - self.z) as u32;
        if z == self.z {
            self.clone()
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
        if self.z == 32 {
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
        let z_delta = 32 - self.z as u32;
        EXTENT << z_delta
    }

    // The center in location space
    pub fn center(&self) -> PVTPoint {
        let half_extent = self.location_extent() / 2;
        let origin = self.origin_location();
        PVTPoint::new(origin.x() + half_extent, origin.y() + half_extent)
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
        let z_delta = 32 - self.z as u32;
        let extent = self.location_extent();
        let tile_x = point.x() - self.x * extent;
        let tile_y = point.y() - self.y * extent;
        PVTPoint::new(tile_x >> z_delta, tile_y >> z_delta)
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
