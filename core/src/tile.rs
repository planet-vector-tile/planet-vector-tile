use fast_hilbert::{xy2h, h2xy};

pub struct Tile {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub h: u64
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

    pub fn hilbert() -> u64 {
        45 as u64
    }

    pub fn hilbert_at_zoom(z: u8) -> u64 {
        54 as u64
    }


}

pub struct HilbertTile {
    pub z: u8,
    pub h: u64
}

impl HilbertTile {
    pub fn new(z: u8, h: u64) -> Self {
        Self { z, h }
    }

    fn index_for_zoom(z: u8) -> u64 {
        z as u64
    }
}

impl From<ZXYTile> for HilbertTile {
    fn from(tile: ZXYTile) -> Self {
        let h = xy2h(tile.x, tile.y);
        Self { z: tile.z, h }
    }
}

pub struct ZXYTile {
    pub z: u8,
    pub x: u32,
    pub y: u32
}

impl ZXYTile {
    pub fn new(z: u8, x: u32, y: u32) -> Self {
        Self { z, x, y }
    }
}

impl From<HilbertTile> for ZXYTile {
    fn from(tile: HilbertTile) -> Self {
        let (x, y) = h2xy(tile.h);
        Self { z: tile.z, x, y }
    }
}

