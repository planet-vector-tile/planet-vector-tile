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
    // This is the coordinate of the tile in location space (zoom 32).
    pub fn origin(&self) -> Tile {
        self.at_zoom(32)
    }

    // The extent of a tile in location space
    pub fn extent(&self) -> u32 {
        let z_delta = (32 - self.z) as u32;
        8192 << z_delta
    }

    pub fn center(&self) -> Tile {
        let extent = self.extent() / 2;
        let origin = self.origin();
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
        let origin = self.origin();
        let extent = self.extent();
        BBox {
            w: origin.x,
            n: origin.y,
            e: origin.x + extent,
            s: origin.y + extent,
        }
    }

    // Projects a point from location space to tile space.
    pub fn project(&self, point: &Point) -> Point {
        let shift = 32 - self.z as u32;
        let extent = self.extent();
        let tile_x = point.x - self.x * extent;
        let tile_y = point.y - self.y * extent;
        Point::new(tile_x >> shift, tile_y >> shift)
    }

    pub fn project_bbox(&self, bbox: &BBox) -> BBox {
        let shift = 32 - self.z as u32;
        let extent = self.extent();
        let w = (bbox.w - self.x * extent) >> shift;
        let n = (bbox.n - self.y * extent) >> shift;
        let e = (bbox.e - self.x * extent) >> shift;
        let s = (bbox.s - self.y * extent) >> shift;
        BBox::new(w, n, e, s)
    }
}

pub struct BBox {
    w: u32,
    n: u32,
    e: u32,
    s: u32,
}

impl BBox {
    pub fn new(w: u32, n: u32, e: u32, s: u32) -> Self {
        BBox { w, n, e, s }
    }

    pub fn nw(&self) -> Point {
        Point::new(self.w, self.n)
    }

    pub fn sw(&self) -> Point {
        Point::new(self.w, self.s)
    }

    pub fn se(&self) -> Point {
        Point::new(self.e, self.s)
    }

    pub fn ne(&self) -> Point {
        Point::new(self.e, self.n)
    }

    pub fn nw(&self) -> Point {
        Point::new(self.w, self.n)
    }
}

// look into using simd
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
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
