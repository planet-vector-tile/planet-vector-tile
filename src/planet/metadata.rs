#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    // Version of PlanetVectorTile that generated data
    version: u32, // string_i

    writing_program: u32, // string_i

    // west, south, east, north
    bbox: [u32; 4],
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct BBox {
    left: u32,
    bottom: u32,
    right: u32,
    top: u32,
}
