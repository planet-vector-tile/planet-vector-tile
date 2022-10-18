use fast_hilbert::{h2xy, xy2h};
use std::ops;

// Look into using simd...
// use std::simd::u32x2;

// The max u32 is 4,294,967,295 (2^32),
// so the unit location tile would be zoom 32.

// In MapboxGL, the max extent for a tile is 8192 (13 bit unsigned integer)
// 4,294,967,296 / 8192 = 524,288
// 2^9 = 524,288 , so zoom 9 and above does not quantize coordinates
// Zooms 8 and below do quantize coordinates.

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
        if z == self.z {
            self.clone()
        } else if z > self.z {
            let shift = (z - self.z) as u32;
            let x = self.x << shift;
            let y = self.y << shift;
            Tile::from_zxy(z, x, y)
        } else {
            let shift = (z - self.z) as u32;
            let x = self.x >> shift;
            let y = self.y >> shift;
            Tile::from_zxy(z, x, y)
        }
    }

    // The origin is the Northwest corner.
    // The location is the mercator xy at zoom 32.
    // You can think of the returned tile as the
    // "Location Tile".
    pub fn origin_location(&self) -> Tile {
        self.at_zoom(32)
    }

    // The extent of a tile on an axis at z32.
    pub fn location_extent(&self) -> u32 {
        let z_delta = (32 - self.z) as u32;
        8192 << z_delta
    }

    pub fn center_location(&self) -> Tile {
        let extent = self.location_extent() / 2;
        let origin = self.origin_location();
        Tile::from_zxy(32, origin.x + extent, origin.y + extent)
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
        let mut desc = vec![self.clone()];
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

    pub fn bbox(&self) -> BBox {
        let origin = self.origin_location();
        let extent = self.location_extent();
        BBox {
            w: origin.x,
            n: origin.y,
            e: origin.x + extent,
            s: origin.y + extent
        }
    }

    pub fn project(&self, point: &Point) {
        let shift = 32 - self.z as u32;
        
    }

}

pub struct BBox {
    w: u32,
    n: u32,
    e: u32,
    s: u32
}

// look into using simd
pub struct Point {
    x: u32,
    y: u32
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point {
            x,
            y
        }
    }
}

impl ops::Add<Point> for Tile {
    type Output = Point;
    fn add(self, _rhs: Point) -> Point {
        Point::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}

impl ops::Sub<Point> for Tile {
    type Output = Point;
    fn sub(self, _rhs: Point) -> Point {
        Point::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}
