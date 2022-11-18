#![allow(dead_code)]

use crate::location;
use crate::mutant::Mutant;
use crate::osmflat::osmflat_generated::osm::{
    HilbertNodePair, HilbertWayPair, Node, NodeIndex, Osm, TagIndex, Way,
};
use crate::tile::{self, Tile};
use flatdata::FileResourceStorage;
use log::info;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::sync::Arc;

// Leaves correspond to additional info we need to know about the tiles at the leaf level.
// We need to know:
//  - The indices into the nodes, ways, relations vectors.
//  - The hilbert index that the given tile starts at.
// Though the hilbert index can be derived from the n,w,r by looking at the hilbert pairs,
// This is referenced often, so this is simpler and saves us from paging into the entity
// (nodes, ways, relations) vectors unnecessarily.
#[derive(Debug)]
pub struct Leaf {
    // Indices to the first node of the given leaf tile.
    pub n: u64,
    pub w: u32,
    pub r: u32,
    // Hilbert index for the leaf tile, at the leaf zoom
    pub h: u32,
}

// Each vector tile corresponds to one of these tiles.
//
// n,w,r are the indices to the entity chunk vectors (n_chunks, w_chunks, r_chunks).
// These are used to retrieve the chunks that tell us what chunks of the entity vectors
// we need to retrieve to construct the given tile.
//
// The levels are descending, with the first level corresponding to the highest zoom,
// in parity with the leaf vector. Each level is z - 2, allowing 16 children per tile.
#[derive(Debug)]
pub struct HilbertTile {
    // Indices to the first chunk of nodes, ways, relations, for the tile.
    pub n: u32,
    pub w: u32,
    pub r: u32,
    // Index of the first child.
    pub child: i32,
    // Bit mask denoting which of the 16 children for the given tile exist.
    // MSB is index 15, MSB is index 0.
    pub mask: u16,
}

// Chunks are offsets and run lengths of entities used for a given tile in the entity.
#[derive(Debug)]
pub struct Chunk {
    pub offset: u32, // offset from the w or r of the leaf tile
    pub len: u32,
}

pub struct HilbertTree {
    pub leaf_zoom: u8,
    pub tiles: Mutant<HilbertTile>,
    pub leaves: Mutant<Leaf>,
    pub n_chunks: Mutant<Chunk>,
    pub w_chunks: Mutant<Chunk>,
    pub r_chunks: Mutant<Chunk>,
    pub archive: Osm,
}

impl HilbertTree {
    pub fn build(dir: &Path, leaf_zoom: u8) -> Result<Self, Box<dyn std::error::Error>> {
        // Leaf zoom must be even
        if leaf_zoom & 1 != 0 {
            eprintln!("leaf zoom not even: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "leaf_zoom must be even!",
            )));
        }

        // Maximum supported zoom is 14.
        if leaf_zoom > 14 {
            eprintln!("Leaf zoom too high! Must be <= 14 z: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Leaf zoom too high! Must be <= 14",
            )));
        }

        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, leaf_zoom)?;
        let m_tiles = build_tiles(&m_leaves, dir, leaf_zoom)?;

        let n_chunks = Mutant::<Chunk>::new(dir, "hilbert_n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "hilbert_w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "hilbert_r_chunks", 1000)?;

        let archive = Osm::open(FileResourceStorage::new(dir))?;

        Ok(Self {
            leaf_zoom,
            tiles: m_tiles,
            leaves: m_leaves,
            n_chunks,
            w_chunks,
            r_chunks,
            archive,
        })
    }

    pub fn open(dir: &Path, leaf_zoom: u8) -> Result<Self, Box<dyn std::error::Error>> {
        // Leaf zoom must be even
        if leaf_zoom & 1 != 0 {
            eprintln!("leaf zoom not even: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "leaf_zoom must be even!",
            )));
        }

        // Maximum supported zoom is 14.
        if leaf_zoom > 14 {
            eprintln!("Leaf zoom too high! Must be <= 14 z: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Leaf zoom too high! Must be <= 14",
            )));
        }

        let m_leaves = Mutant::<Leaf>::open(dir, "hilbert_leaves", false)?;
        let m_tiles = Mutant::<HilbertTile>::open(dir, "hilbert_tiles", false)?;

        let n_chunks = Mutant::<Chunk>::new(dir, "hilbert_n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "hilbert_w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "hilbert_r_chunks", 1000)?;

        let archive = Osm::open(FileResourceStorage::new(dir))?;

        Ok(Self {
            leaf_zoom,
            tiles: m_tiles,
            leaves: m_leaves,
            n_chunks,
            w_chunks,
            r_chunks,
            archive,
        })
    }
}

fn build_leaves(
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<Mutant<Leaf>, Box<dyn std::error::Error>> {
    let node_pairs = m_node_pairs.slice();
    let way_pairs = m_way_pairs.slice();

    if node_pairs.len() == 0 && way_pairs.len() == 0 {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            "No hilbert pairs found! Cannot build hilbert tiles.",
        )));
    }

    let mut n_i: usize = 0; // node hilbert pair index
    let mut w_i: usize = 0; // way hilbert pair index

    let mut lowest_h = 0;

    // Find the lowest hilbert tile
    if let Some(first_node_pair) = node_pairs.first() {
        lowest_h = first_node_pair.h();
        n_i = 1;
    }
    if let Some(first_way_pair) = way_pairs.first() {
        let first_way_h = first_way_pair.h();
        if first_way_h < lowest_h {
            lowest_h = first_way_h;
            w_i = 1;
            n_i = 0;
        }
    }

    // First leaf Hilbert tile has the lowest hilbert location.
    let mut tile_h = location::h_to_zoom_h(lowest_h, leaf_zoom) as u32;
    info!(
        "lowest tile_h for leaves in hilbert tree: {}, leaf_zoom: {}",
        lowest_h, leaf_zoom
    );

    // NHTODO Implement the ability to grow the LeafTile mutant so that we don't have to allocate max size upfront?
    let max_len = tile::tile_count_for_zoom(leaf_zoom) as usize;
    let mut m_leaves = Mutant::<Leaf>::new(dir, "hilbert_leaves", max_len)?;
    let leaves = m_leaves.mutable_slice();

    let node_pairs = m_node_pairs.slice();
    let node_pairs_len = node_pairs.len();
    let way_pairs = m_way_pairs.slice();
    let way_pairs_len = way_pairs.len();

    // First leaf tile
    leaves[0] = Leaf {
        n: 0,
        w: 0,
        r: 0,
        h: tile_h,
    };

    let mut leaf_i = 1;

    let mut node_tile_h = tile_h;
    let mut way_tile_h = tile_h;

    let archive = Osm::open(FileResourceStorage::new(dir))?;
    let ways = archive.ways();

    loop {
        let mut next_tile_h = None;

        while n_i < node_pairs_len && node_tile_h == tile_h {
            let p = &node_pairs[n_i];
            let node_h = p.h();
            node_tile_h = location::h_to_zoom_h(node_h, leaf_zoom) as u32;
            if node_tile_h > tile_h {
                next_tile_h = Some(node_tile_h);
                break;
            }
            n_i += 1;
        }

        while w_i < way_pairs_len && way_tile_h == tile_h {
            let p = &way_pairs[w_i];
            let way_h = p.h();
            way_tile_h = location::h_to_zoom_h(way_h, leaf_zoom) as u32;
            let way_id = ways[w_i].osm_id();
            if way_id == 42630986 {
                println!("way_id: {}, way_tile_h: {} tile_h {}", way_id, way_tile_h, tile_h);
            }
            if way_tile_h > tile_h {
                if way_tile_h < node_tile_h {
                    next_tile_h = Some(way_tile_h);
                }
                break;
            }
            w_i += 1;
        }

        if let Some(next_tile_h) = next_tile_h {
            leaves[leaf_i] = Leaf {
                n: n_i as u64,
                w: w_i as u32,
                r: 0,
                h: next_tile_h,
            };
            tile_h = next_tile_h;
            leaf_i += 1;
        } else {
            break;
        }
    }

    // The last increment of t_i falls through both whiles, so it is equal to the length.
    m_leaves.set_len(leaf_i);
    m_leaves.trim();
    Ok(m_leaves)
}

fn build_tiles(
    m_leaves: &Mutant<Leaf>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<Mutant<HilbertTile>, Box<dyn std::error::Error>> {
    let max_tiles_len = max_tiles_len(&m_leaves, leaf_zoom);
    let mut m_tiles = Mutant::<HilbertTile>::new(dir, "hilbert_tiles", max_tiles_len)?;
    let tiles = m_tiles.mutable_slice();

    let mut z = leaf_zoom;
    let mut tiles_i = 0;

    println!("zoom {} level_start {}", leaf_zoom, 0);

    let leaves = m_leaves.slice();
    for _ in leaves {
        // The default leaf has a 0 tile mask,
        // which is needed to know we are at a leaf tile.
        let t = HilbertTile::default();
        println!(
            "tiles_i {} h {} mask {:#018b} {:?}",
            tiles_i, leaves[tiles_i].h, t.mask, t
        );
        tiles[tiles_i] = t;
        tiles_i += 1;
    }

    let mut level_range = 0..tiles_i;
    z -= 2;

    loop {
        let level_start = tiles_i;
        println!("zoom {} level_start {}", z, level_start);

        // The first child for the tile we are building.
        let mut child_i = level_range.start;

        while child_i < level_range.end {
            // The tile we are building.
            let leaf_h = get_leaf_h(child_i, tiles, leaves);
            let tile_h = leaf_to_tile_h(leaf_h, leaf_zoom, z);
            let h_range_end = child_h_range_end(tile_h);

            let mut mask: u16 = 0;

            // First child for level
            let mut child_h = get_child_h(child_i, leaf_zoom, z + 2, tiles, leaves);

            // Position of the possible children of the tile. 0 -> 16
            let child_pos = (child_h & 0xf) as u16;
            let child_bit = 1 << child_pos;
            mask |= child_bit;

            // first child_i with offset
            let child = child_i as i32 - child_pos as i32;

            child_h = get_child_h(child_i, leaf_zoom, z + 2, tiles, leaves);
            child_i += 1;

            while child_h < h_range_end && child_i < level_range.end {
                // Position of the possible children of the tile. 0 -> 16
                let child_pos = (child_h & 0xf) as u16;
                let child_bit = 1 << child_pos;
                mask |= child_bit;

                child_h = get_child_h(child_i, leaf_zoom, z + 2, tiles, leaves);
                child_i += 1;
            }

            let mut t = HilbertTile::default();
            t.child = child;
            t.mask = mask;
            println!("{} h {} mask {:#018b} {:?}", tiles_i, tile_h, t.mask, t);
            tiles[tiles_i] = t;

            tiles_i += 1;
        }

        level_range = level_start..tiles_i;

        if z == 0 {
            break;
        }
        z -= 2;
    }

    m_tiles.set_len(tiles_i);
    m_tiles.trim();

    Ok(m_tiles)
}

fn max_tiles_len(m_leaves: &Mutant<Leaf>, leaf_zoom: u8) -> usize {
    let leaves = m_leaves.slice();
    if leaves.len() == 0 {
        0
    } else if leaves.len() == 1 {
        (leaf_zoom / 2) as usize
    } else {
        let len = leaves.len();
        let first = leaves[0].h;
        let last = leaves[len - 1].h;
        let mut potential_leaves = last - first + 1;

        let mut count = potential_leaves;
        let mut zoom = leaf_zoom - 2;
        while zoom > 0 {
            potential_leaves = potential_leaves / 4;
            count += potential_leaves;
            zoom -= 2;
        }
        (count + 1) as usize
    }
}

const CHILD_POSITIONS: [u16; 16] = [
    0b0000000000000001,
    0b0000000000000010,
    0b0000000000000100,
    0b0000000000001000,
    0b0000000000010000,
    0b0000000000100000,
    0b0000000001000000,
    0b0000000010000000,
    0b0000000100000000,
    0b0000001000000000,
    0b0000010000000000,
    0b0000100000000000,
    0b0001000000000000,
    0b0010000000000000,
    0b0100000000000000,
    0b1000000000000000,
];

fn get_leaf_h(tiles_i: usize, tiles: &[HilbertTile], leaves: &[Leaf]) -> u32 {
    let mut i = tiles_i;
    let mut tile = &tiles[tiles_i];
    while tile.mask != 0 {
        let pos = first_child_pos(tile.mask) as i32;
        i = (tile.child + pos) as usize;
        tile = &tiles[i];
    }
    leaves[i].h
}

fn leaf_to_tile_h(h: u32, leaf_zoom: u8, zoom: u8) -> u32 {
    h >> (2 * (leaf_zoom - zoom))
}

fn get_child_h(
    tiles_idx: usize,
    leaf_zoom: u8,
    zoom: u8,
    tiles: &[HilbertTile],
    leaves: &[Leaf],
) -> u32 {
    let leaf_h = get_leaf_h(tiles_idx, tiles, leaves);
    leaf_to_tile_h(leaf_h, leaf_zoom, zoom)
}

fn child_h_range_end(h: u32) -> u32 {
    let start = h << 4;
    start + 16
}

impl HilbertTile {
    fn default() -> Self {
        Self {
            n: 0,
            w: 0,
            r: 0,
            child: 0,
            mask: 0,
        }
    }
}

fn first_child_pos(mask: u16) -> u8 {
    for i in 0..16 {
        if (mask >> i) & 1 == 1 {
            return i;
        }
    }
    0
}

fn mask_has_children(mask: u16) -> bool {
    mask != 0
}

fn mask_include(mask: u16, child_idx: u8) -> u16 {
    mask | 1 << child_idx
}

fn mask_has(mask: u16, child_pos: u8) -> bool {
    (mask >> child_pos & 1) == 1
}

fn level_path(tile: Tile) -> Vec<u8> {
    let mut h = tile.h;
    let mut z = tile.z;
    let mut path: Vec<u8> = Vec::new();

    while z > 0 {
        let child_pos = (h & 0xf) as u8;
        path.push(child_pos);
        h = h >> 4;
        z -= 2;
    }
    path
}

// https://doc.rust-lang.org/nomicon/other-reprs.html
// https://adventures.michaelfbryan.com/posts/deserializing-binary-data-files/
// https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
// We don't actually need to use this, but it is helpful for tests.
// The mutant memmap vectors get allocated in bulk, and they are effectively this on disk.
// No serde is necessary, due to the memmap mechanism.
unsafe fn to_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::PathBuf};

    use crate::source::Source;

    use super::*;

    #[test]
    fn test_struct_binary() {
        let c = Chunk {
            offset: 0xABCDEF33,
            len: 0x87654321,
        };

        unsafe {
            let bytes = to_bytes(&c);
            let str = format!("{:x?}", bytes);
            assert_eq!(str, "[33, ef, cd, ab, 21, 43, 65, 87]")
        }

        let p = PathBuf::from("/Users/n/tmp");
        let chunks = Mutant::<Chunk>::new(&p, "test", 1000).unwrap();

        let s = chunks.mutable_slice();
        let s0 = &mut s[0];
        s0.offset = 0x11111111;
        s0.len = 0x22222222;

        let slc2 = chunks.slice();
        let s2 = &slc2[0];

        let str2 = format!("{:x?}", unsafe { to_bytes(s2) });
        assert_eq!(str2, "[11, 11, 11, 11, 22, 22, 22, 22]");
    }

    #[test]
    fn test_mask() {
        let mask = 0;

        assert_eq!(mask_has_children(mask), false);

        let m2 = mask_include(0, 5);
        assert_eq!(0b100000, m2);

        let m3 = mask_include(m2, 15);
        assert_eq!(0b1000000000100000, m3);

        assert!(!mask_has(m3, 0));

        assert!(mask_has(m3, 5));
        assert!(mask_has(m3, 15));
    }

    #[test]
    fn test_4nodes_leaf_tiles() {
        let dir = PathBuf::from("./tests/fixtures/nodes4/sort");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let node_pairs = m_node_pairs.slice();
        let mut leaf_tiles = HashSet::<u32>::new();
        for p in node_pairs {
            let zoom_h = location::h_to_zoom_h(p.h(), 12) as u32;
            leaf_tiles.insert(zoom_h);
            // println!("{:?} zoom_h: {}", p, zoom_h);
        }
        // Should be a total of 3 unique tiles for the 4 nodes.
        assert_eq!(leaf_tiles.len(), 3);
    }

    #[test]
    fn test_build_leaves() {
        let dir = PathBuf::from("./tests/fixtures/nodes4/sort");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, 12).unwrap();

        // We know there are 3 unique leaf tiles for the 4 nodes.
        assert_eq!(m_leaves.len, 3);
    }

    #[test]
    fn test_santacruz() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, 12).unwrap();

        assert_eq!(m_leaves.len, 189);

        let leaves = m_leaves.slice();
        for l in leaves {
            let h = l.h;
            println!("leaf tile h {:?}", h);
            let leaf_zoom = 12;
            let z = 10;
            let parent_h = h >> (2 * (leaf_zoom - z));
            println!("leaf parent h {:?}", parent_h);
        }

        let tree = HilbertTree::build(&dir, 12).unwrap();
        // let m_tiles = tree.tiles;
        // let tiles = m_tiles.slice();
        // for t in tiles {
        //     println!("{:?}", t);
        // }

        // let vec_u8 = tree.compose_tile(Tile::from_zh(12, 3329090));
        // let vec_u8 = tree.compose_tile(Tile::from_zh(12, 3329140));
    }

    // #[test]
    // fn test_level_path() {
    //     let tile = Tile::from_zh(12, 3329121);
    //     let path = level_path(tile);
    //     println!("path {:?}", path);
    //     // [1, 6, 12, 12, 2, 3]

    //     let mut t = Tile::default();
    //     for p in path.iter().rev() {
    //         let mut grand_children = t.grand_children();
    //         grand_children.sort_by_key(|c| c.h);
    //         t = grand_children[*p as usize];
    //     }
    //     assert_eq!(t, tile);
    // }
}
