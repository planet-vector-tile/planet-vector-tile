use super::{
    hilbert_tile::{Chunk, HilbertTile},
    leaf::Leaf,
};
use crate::mutant::Mutant;
use std::path::Path;

pub fn build_chunks(
    m_leaves: &Mutant<Leaf>,
    m_tiles: &Mutant<HilbertTile>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<(Mutant<Chunk>, Mutant<Chunk>, Mutant<Chunk>), Box<dyn std::error::Error>> {
    let leaves = m_leaves.slice();
    let tiles = m_tiles.mutable_slice();

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;
    let mut first = true;
    for tile in tiles {

        // filter children features



        children += count_children(tile.mask);
        // If we are done with the level, decrement z to the next zoom.
        if (tile.child == 0 && !first) || children == total_children {
            total_children = level_tile_count;
            level_tile_count = 0;
            z -= 2;
        } else {
            level_tile_count += 1;
            first = false;
        }
    }

    let n_chunks = Mutant::<Chunk>::open(dir, "hilbert_node_chunks", false)?;
    let w_chunks = Mutant::<Chunk>::open(dir, "hilbert_way_chunks", false)?;
    let r_chunks = Mutant::<Chunk>::open(dir, "hilbert_relation_chunks", false)?;

    Ok((n_chunks, w_chunks, r_chunks))
}

fn count_children(mask: u16) -> u32 {
    let mut count = 0;
    for i in 0..16 {
        if mask >> i & 1 == 1 {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_chunks() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let m_leaves = Mutant::<Leaf>::open(&dir, "hilbert_leaves", false).unwrap();
        let m_tiles = Mutant::<HilbertTile>::open(&dir, "hilbert_tiles", false).unwrap();
        let _ = build_chunks(&m_leaves, &m_tiles, &dir, 12);
    }
}
