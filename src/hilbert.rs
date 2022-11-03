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

const MB_500: usize = 524288000;

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

        let tree = Mutant::<NodeTile>::new(dir, "pyramid", 1000)?;
        // let leaves = Mutant::<LeafTile>::new(dir, "leaves", 1000)?;
        let n_chunks = Mutant::<Chunk>::new(dir, "n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "r_chunks", 1000)?;
        
    

        let path = dir.join("leaves");
        let leaves_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let leaves_writer = BufWriter::with_capacity(MB_500, leaves_file);

        let nodes_mut = Mutant::<Node>::open(dir, "nodes")?;
        let hilbert_node_pairs_mut = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs")?;
        let hilbert_node_pairs = hilbert_node_pairs_mut.slice();
        
        let first_node_h = hilbert_node_pairs[0].h();
        let mut t = Tile::from_zh(32, first_node_h);
        let mut h_range = t.h_range_for_zoom(leaf_zoom);
        

        for (i, pair) in hilbert_node_pairs[1..].iter().enumerate() {
            let h = pair.h();
            if h < h_range.end {

            }
        }



        let leaves = Mutant::<LeafTile>::open(dir, "leaves")?;

        Ok(Self {
            leaf_zoom,
            tree: Cell::new(tree),
            leaves: Cell::new(leaves),
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

struct LeafTile {
    n: u64,
    w: u32,
    r: u32,
    chunks: Chunks,
}

struct NodeTile {
    children: [u32; 16], // offsets from current index
    chunks: Chunks,
}

struct Chunks {
    n: u32,
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


}

