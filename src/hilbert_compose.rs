use crate::{hilbert::{HilbertTree, HilbertTile, Leaf}, tile::Tile, source::Source};

struct Result<'a> {
    h_tile: &'a HilbertTile,
    next_h_tile: Option<&'a HilbertTile>,
    leaf: Option<&'a Leaf>,
    next_leaf: Option<&'a Leaf>,
}

impl HilbertTree {
    fn find(&self, tile: Tile) -> Option<Result> {
        if tile.z > self.leaf_zoom {
            return None;
        }

        println!("Finding {:?}", tile);

        let h_tiles = self.tiles.slice();
        let mut h_tile = h_tiles.last().unwrap();
        println!("root {:?}", h_tile);

        let mut z = 2;
        let mut i = 0;
        while z <= tile.z {
            let h = tile.h >> (2 * (tile.z - z));
            let child_pos = (h & 0xf) as i32;

            // If the tile does not have the child position in the mask,
            // then we don't have the tile.
            if h_tile.mask >> child_pos & 1 != 1 {
                return None;
            }

            i = (h_tile.child + child_pos) as usize;

            h_tile = &h_tiles[i];

            println!("i {} {:?}", i, h_tile);

            z += 2;
        }

        let next_h_tile = if i + 1 < h_tiles.len() {
            Some(&h_tiles[i + 1])
        } else {
            None
        };

        let mut leaf = None;
        let mut next_leaf = None;
        if h_tile.mask == 0 {
            let leaves = self.leaves.slice();
            leaf = Some(&leaves[i]);
            if i + 1 < leaves.len() {
                next_leaf = Some(&leaves[i + 1]);
            }
        }

        Some( Result {
            h_tile,
            next_h_tile: None,
            leaf,
            next_leaf: None,
        })
    }

    pub fn hello(&self) -> i32 {
        234
    }

    pub fn compose_tile(&self, tile: Tile) -> Vec<u8> {
        match self.find(tile) {
            Some(res) => {
                if let Some(leaf) = res.leaf {

                    print!("leaf found {:?}", leaf);

                    let nodes = self.archive.nodes();
                    let ways = self.archive.ways();
                    let relations = self.archive.relations();

                    let (n_end, w_end, r_end) = if let Some(next_leaf) = res.next_leaf {
                        (next_leaf.n, next_leaf.w, next_leaf.r)
                    } else {
                        (nodes.len() as u64, ways.len() as u32, relations.len() as u32)
                    };

                    



                }

                Vec::new()
            }
            None => Vec::new(),
        }
    }
}

impl Source for HilbertTree {
    fn tile(&self, tile: &Tile) -> Vec<u8> {
        vec![1,2,3,4,222,55]
    }
}
