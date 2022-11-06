#![allow(dead_code)]

use crate::location;
use crate::mutant::Mutant;
use crate::osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair};
use crate::tile;
use log::info;
use std::cell::Cell;
use std::io::{Error, ErrorKind};
use std::path::Path;

// 4^14 = 268,435,456
// 4^16 = 4,294,967,296
// 2^32 = 4,294,967,296

pub struct HilbertTree {
    leaf_zoom: u8,
    tree: Cell<Mutant<NodeTile>>,
    leaves: Cell<Mutant<LeafTile>>,
    n_chunks: Cell<Mutant<Chunk>>,
    w_chunks: Cell<Mutant<Chunk>>,
    r_chunks: Cell<Mutant<Chunk>>,
}

impl HilbertTree {
    pub fn build(dir: &Path, leaf_zoom: u8) -> Result<Self, Box<dyn std::error::Error>> {
        // Leaf zoom must be even
        if leaf_zoom & 1 != 0 {
            eprintln!("leaf zoom not even: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Leaf zoom must be even!",
            )));
        }

        // To save space, we store the hilbert values in a u32,
        // so the highest zoom guaranteeing no overflow is 16.
        if leaf_zoom > 16 {
            eprintln!("Leaf zoom too high! Must be <= 16 z: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Leaf zoom too high! Must be <= 16",
            )));
        }

        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;
        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, leaf_zoom)?;

        let tree = Mutant::<NodeTile>::new(dir, "hilbert_tree", 1000)?;
        let n_chunks = Mutant::<Chunk>::new(dir, "hilbert_n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "hilbert_w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "hilbert_r_chunks", 1000)?;

        Ok(Self {
            leaf_zoom,
            tree: Cell::new(tree),
            leaves: Cell::new(m_leaves),
            n_chunks: Cell::new(n_chunks),
            w_chunks: Cell::new(w_chunks),
            r_chunks: Cell::new(r_chunks),
        })
    }
}

fn build_leaves(
    m_node_pairs: &Mutant<HilbertNodePair>,
    m_way_pairs: &Mutant<HilbertWayPair>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<Mutant<LeafTile>, Box<dyn std::error::Error>> {
    let node_pairs = m_node_pairs.slice();
    let way_pairs = m_way_pairs.slice();

    if node_pairs.len() == 0 && way_pairs.len() == 0 {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            "No hilbert pairs found! Cannot build hilbert tiles.",
        )));
    }

    let mut t_i: usize = 0; // tile index
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

    info!("lowest_h for leaves in hilbert tree: {}", lowest_h);

    // First leaf Hilbert tile has the lowest hilbert location.
    let mut tile_h = location::h_to_zoom_h(lowest_h, leaf_zoom) as u32;
    info!("lowest tile_h for leaves in hilbert tree: {}, leaf_zoom: {}", lowest_h, leaf_zoom);

    // NHTODO Implement the ability to grow the LeafTile mutant so that we don't have to allocate max size upfront?
    let max_len = tile::tile_count_for_zoom(leaf_zoom) as usize;
    let mut m_leaves = Mutant::<LeafTile>::new(dir, "hilbert_leaves", max_len)?;
    let leaves = m_leaves.mutable_slice();

    let node_pairs = m_node_pairs.slice();
    let node_pairs_len = node_pairs.len();
    let way_pairs = m_way_pairs.slice();
    let way_pairs_len = way_pairs.len();

    // First tile
    leaves[t_i] = LeafTile {
        first_entity_idx: NWR { n: 0, w: 0, r: 0 },
        first_chunk_idx: NWRChunk { n: 0, w: 0, r: 0 },
        tile_h,
    };

    let mut node_tile_h = tile_h;
    let mut way_tile_h = tile_h;

    while n_i < node_pairs_len || w_i < way_pairs_len {
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
            if way_tile_h > tile_h {
                if way_tile_h < node_tile_h {
                    next_tile_h = Some(way_tile_h);
                }
                break;
            }
            w_i += 1;
        }

        if let Some(next_tile_h) = next_tile_h {
            tile_h = next_tile_h;
        } else {
            break;
        }

        t_i += 1;
    }

    m_leaves.set_len(t_i);
    m_leaves.trim();
    Ok(m_leaves)
}

struct NWR {
    n: u64,
    w: u32,
    r: u32,
}

struct LeafTile {
    first_entity_idx: NWR,
    first_chunk_idx: NWRChunk,
    tile_h: u32, // At the leaf zoom
}

struct NodeTile {
    first_chunk_idx: NWRChunk,
    children_idx: u32,
    children_mask: u16,
}

fn mask_has_children(mask: u16) -> bool {
    mask != 0
}

fn mask_include(mask: u16, child_idx: u8) -> u16 {
    mask | 1 << child_idx
}

fn mask_has(mask: u16, child_idx: u8) -> bool {
    (mask >> child_idx & 1) == 1
}

struct NWRChunk {
    n: u32, // offset, so it doesn't need to be u64
    w: u32,
    r: u32,
}

struct Chunk {
    offset: u32, // offset from the w or r of the leaf tile
    len: u32,
}

// https://doc.rust-lang.org/nomicon/other-reprs.html
// https://adventures.michaelfbryan.com/posts/deserializing-binary-data-files/
// https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8

unsafe fn to_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, collections::HashSet};

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
        let dir = PathBuf::from("./test/fixtures/4nodes/archive");
        let m_node_pairs = Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
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
        let dir = PathBuf::from("./test/fixtures/4nodes/archive");
        let m_node_pairs =
            Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, 12).unwrap();

        // We know there are 3 unique leaf tiles for the 4 nodes.
        assert_eq!(m_leaves.len, 3);
    }
}
