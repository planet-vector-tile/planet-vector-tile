#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    // Version of PlanetVectorTile that generated data
    version: u32, // string_i

    // Info about the source, ie: OpenStreetMap Planet PBF or URL.
    source: u32, // string_i

    // west, south, east, north
    bbox: [u32; 4],

    osm: OSMMetadata,
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct BBox {
    left: u32,
    bottom: u32,
    right: u32,
    top: u32,
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct OSMMetadata {
    replication_timestamp: i64,
    sequence_number: i64,
    replication_base_url: u32, // string_i
}
