use crate::{hilbert::{HilbertTree, HilbertTile, Leaf}, tile::Tile};

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

        let mut leaf = None;
        if h_tile.mask == 0 {
            let leaves = self.leaves.slice();
            leaf = Some(&leaves[i]);
        }

        Some( Result {
            h_tile,
            next_h_tile: None,
            leaf,
            next_leaf: None,
        })
    }

    pub fn compose_tile(&self, tile: Tile) -> Vec<u8> {
        match self.find(tile) {
            Some((_, leaf)) => {
                if let Some(leaf) = leaf {
                    print!("leaf found {:?}", leaf);
                }

                Vec::new()
            }
            None => Vec::new(),
        }
    }
}
