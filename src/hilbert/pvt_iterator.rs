use crate::{pvt_builder::PVTBuilder, tile::Tile};

use super::{tree::{HilbertTree, ResultPair}, leaf::Leaf};

impl HilbertTree {
    pub fn pvt_leaf_iterator(&self) -> PVTLeafIterator {
        PVTLeafIterator::new(self)
    }
}

pub struct PVTLeafIterator<'a> {
    tree: &'a HilbertTree,
    leaves: &'a [Leaf],
    leaf_zoom: u8,
    i: usize,
}

impl<'a> PVTLeafIterator<'a> {
    pub fn new(tree: &'a HilbertTree) -> Self {
        PVTLeafIterator {
            tree,
            leaves: tree.leaves.slice(),
            leaf_zoom: tree.manifest.render.leaf_zoom,
            i: 0,
        }
    }
}

impl<'a> Iterator for PVTLeafIterator<'a> {
    type Item = (Tile, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        // next
        let leaf = if self.i < self.leaves.len() {
            let leaf = &self.leaves[self.i];
            self.i += 1;
            leaf
        } else {
            return None;
        };

        // next next
        let next_leaf = if self.i < self.leaves.len() {
            Some(&self.leaves[self.i])
        } else {
            None
        };

        let result_pair = ResultPair {
            item: leaf,
            next: next_leaf,
        };
        let h = leaf.h as u64;
        let tile = Tile::from_zh(self.leaf_zoom, h);
        let mut builder = PVTBuilder::new();

        self.tree.compose_leaf(&tile, result_pair, &mut builder);
        let vec_u8 = builder.build();
        Some((tile, vec_u8))
    }
}
