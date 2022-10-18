use fast_hilbert::{h2xy, xy2h};

#[allow(dead_code, unused_imports)]
#[path = "./fbs/planet_vector_tile_generated.rs"]
mod planet_vector_tile_generated;
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

#[derive(Debug, Copy, Clone)]
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

    pub fn descendants(&self, levels: u8) -> Vec<Tile> {
        let mut desc = vec![];
        if levels == 0 {
            return desc;
        }
        let mut children = Vec::from(self.children());
        desc.append(&mut children);
        let mut q = children.clone();
        for tile in q.pop() {
            let children = tile.children();
            desc.append(&mut Vec::from(children));
            if tile.z <= self.z + levels {
                q.append(&mut Vec::from(children));
            }
        }
        desc
    }

    pub fn tree(&self, levels: u8) -> Vec<Tile> {
        self.descendants(levels)
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

pub struct BBox {
    // min
    nw: PVTPoint,
    // max
    se: PVTPoint
}
