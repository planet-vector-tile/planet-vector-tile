#![allow(dead_code)]

use crate::mutant::Mutant;
use crate::osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair};
use std::cell::Cell;
use std::io::{Error, ErrorKind};
use std::path::Path;

// 4^14 = 268,435,456
// 4^16 = 4,294,967,296
// 2^32 = 4,294,967,296

pub struct HilbertTiles {
    leaf_zoom: u8,
    tree: Cell<Mutant<NodeTile>>,
    leaves: Cell<Mutant<LeafTile>>,
    n_chunks: Cell<Mutant<Chunk>>,
    w_chunks: Cell<Mutant<Chunk>>,
    r_chunks: Cell<Mutant<Chunk>>,
}

impl HilbertTiles {
    pub fn build(dir: &Path, leaf_zoom: u8) -> Result<Self, Box<dyn std::error::Error>> {
        // Leaf zoom must be even
        if leaf_zoom & 1 != 0 {
            eprintln!("leaf zoom not even: {}", leaf_zoom);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Leaf zoom must be even!",
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

    // Find the lowest hilbert tile
    // NHTODO Yuck, how do I do this nicely in Rust?
    let mut lowest_h_opt = None;
    if let Some(first_node_pair) = node_pairs.first() {
        lowest_h_opt = Some(first_node_pair.h());
    }
    if let Some(first_way_pair) = way_pairs.first() {
        let first_way_h = first_way_pair.h();
        if let Some(lh) = lowest_h_opt {
            if first_way_h < lh {
                lowest_h_opt = Some(first_way_h);
            }
        }
    }

    if lowest_h_opt.is_none() {
        return Err(Box::new(Error::new(
            ErrorKind::NotFound,
            "No hilbert pairs found!",
        )));
    }
    let lowest_h = lowest_h_opt.unwrap();

    println!("lowest_h {}", lowest_h);

    let mut tile_h = zoom_h(lowest_h, 32, leaf_zoom);
    let mut t_i: usize = 0;
    let mut n_i: usize = 1;
    let mut w_i: usize = 1;

    let mut m_leaves = Mutant::<LeafTile>::new(dir, "hilbert_leaves", 10_000_000)?;
    let leaves = m_leaves.mutable_slice();
    let leaves_len = leaves.len();
    let node_pairs = m_node_pairs.slice();
    let node_pairs_len = node_pairs.len();
    let way_pairs = m_way_pairs.slice();
    let way_pairs_len = way_pairs.len();
    while t_i < leaves_len && (n_i < node_pairs_len || w_i < way_pairs_len) {
        leaves[t_i] = LeafTile {
            first_entity_idx: NWR {
                n: n_i as u64,
                w: w_i as u32,
                r: 0,
            },
            first_chunk_idx: NWRChunk { n: 0, w: 0, r: 0 },
            tile_h,
        };

        let mut node_tile_h = tile_h;
        let mut way_tile_h = tile_h;

        while n_i < node_pairs_len {
            let p = &node_pairs[n_i];
            let node_h = p.h();
            node_tile_h = zoom_h(node_h, 32, leaf_zoom);
            if node_tile_h > tile_h {
                break;
            }
            n_i += 1;
        }

        while w_i < way_pairs_len {
            let p = &way_pairs[w_i];
            let way_h = p.h();
            way_tile_h = zoom_h(way_h, 32, leaf_zoom);
            if way_tile_h > tile_h {
                break;
            }
            w_i += 1;
        }

        if node_tile_h > tile_h && node_tile_h < way_tile_h {
            tile_h = node_tile_h;
        } else {
            tile_h = way_tile_h;
        }

        t_i += 1;
    }

    m_leaves.set_len(t_i);
    m_leaves.trim();
    Ok(m_leaves)
}

fn zoom_h(h: u64, from_z: u8, to_z: u8) -> u32 {
    (h >> 2 * (from_z - to_z)) as u32
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
    use std::path::PathBuf;

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
    fn test_build_leaves() {
        let dir = PathBuf::from("./test/fixtures/4nodes/archive");
        let m_node_pairs = Mutant::<HilbertNodePair>::open(&dir, "hilbert_node_pairs", true).unwrap();
        let m_way_pairs = Mutant::<HilbertWayPair>::open(&dir, "hilbert_way_pairs", true).unwrap();

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, 12).unwrap();

        assert_eq!(m_leaves.len, 3);
    }

}
