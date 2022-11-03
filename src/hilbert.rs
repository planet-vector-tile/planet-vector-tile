use std::cell::Cell;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::io::BufWriter;

use crate::tile::Tile;
use crate::mutant::Mutant;
use crate::osmflat::osmflat_generated::osm::{Node, HilbertNodePair, Way, HilbertWayPair, Relation, HilbertRelationPair};

// 4^14 = 268,435,456
// 4^16 = 4,294,967,296
// 2^32 = 4,294,967,296

struct HilbertTiles {
    leaf_zoom: u8,
    tree: Cell<Mutant<NodeTile>>,
    leaves: Cell<Mutant<LeafTile>>,
    n_chunks: Cell<Mutant<Chunk>>,
    w_chunks: Cell<Mutant<Chunk>>,
    r_chunks: Cell<Mutant<Chunk>>,
}

impl HilbertTiles {
    pub fn new(dir: &Path, leaf_zoom: u8) -> Result<Self, Box<dyn std::error::Error>> {
        
        // Leaf zoom must be even
        if leaf_zoom & 1 == 0 {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "No hilbert node pairs!",
            )));
        }

        let tree = Mutant::<NodeTile>::new(dir, "hilbert_tree", 1000)?;
        let m_leaves = Mutant::<LeafTile>::new(dir, "leaves", 100000)?;
        let n_chunks = Mutant::<Chunk>::new(dir, "n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "r_chunks", 1000)?;
        
        let m_nodes = Mutant::<Node>::open(dir, "nodes", true)?;
        let m_ways = Mutant::<Way>::open(dir, "ways", true)?;
        let m_relations = Mutant::<Relation>::open(dir, "relations", true)?;
        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;

        let node_pairs = m_node_pairs.slice();
        let way_pairs = m_way_pairs.slice();

        // Find the lowest hilbert tile
        
        let mut lowest_h = node_pairs.first().unwrap().h();
        let way_h = way_pairs.first().unwrap().h();
        if way_h < lowest_h {
            lowest_h = way_h;
        }

        let mut tile_h = idx_h_to_zoom(lowest_h, leaf_zoom);
        let mut t_i: usize = 0;
        let mut n_i: usize = 1;
        let mut w_i: usize = 1;

        let leaves = m_leaves.mutable_slice();
        leaves[t_i] = LeafTile {
            first_entity_idx: NWR {
                n: 0,
                w: 0,
                r: 0,
            },
            first_chunk_idx: NWRChunk {
                n: 0,
                w: 0,
                r: 0,
            },
            tile_h,
        };

        let node_pairs = m_node_pairs.slice();
        let node_pairs_len = node_pairs.len();
        while n_i < node_pairs_len {
            let p = &node_pairs[n_i];
            let node_h = p.h();
            let node_tile_h = idx_h_to_zoom(node_h, leaf_zoom);
            if node_tile_h > tile_h {
                t_i += 1;
                tile_h = node_tile_h;
                leaves[t_i].tile_h = node_tile_h;
                leaves[t_i].first_entity_idx.n = n_i as u64;
            }
            n_i += 1;
        }
  
        let way_pairs = m_way_pairs.slice();
        let way_pairs_len = way_pairs.len();
        while w_i < way_pairs_len {
            let p = &way_pairs[w_i];
            let way_h = p.h();
            let way_tile_h = idx_h_to_zoom(way_h, leaf_zoom);

        }

        Ok(Self {
            leaf_zoom,
            tree: Cell::new(tree),
            leaves: Cell::new(m_leaves),
            n_chunks: Cell::new(n_chunks),
            w_chunks: Cell::new(w_chunks),
            r_chunks: Cell::new(r_chunks),
        })
    }

    // pub fn open(dir: &Path, file_name: &str) -> Result<Self> {
      
    // }

    // pub fn build(&self) {

    // }

}

fn idx_h_to_zoom(h: u64, z: u8) -> u32 {
    (h >> 2 * (32 - z)) as u32
}

struct NWR {
    n: u64,
    w: u32,
    r: u32
}

struct LeafTile {
    first_entity_idx: NWR,
    first_chunk_idx: NWRChunk,
    tile_h: u32, // a the leaf zoom
}

struct NodeTile {
    first_chunk_idx: NWRChunk,
    children_idx: u32,
    children_mask: u16,
}

impl NodeTile {

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
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}


mod tests {
    use std::path::PathBuf;

    #[allow(unused_imports)]
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

        let str2 = format!("{:x?}", unsafe { to_bytes(s2)});
        assert_eq!(str2, "[11, 11, 11, 11, 22, 22, 22, 22]");
        println!("{:x?}", unsafe { to_bytes(s0) } );
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

}

