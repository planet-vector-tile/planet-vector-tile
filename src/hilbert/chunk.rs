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

    // for t in tiles {
    //     println!("t {:?}", t);
    // }

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;
    for tile in tiles {
        children += count_children(tile.mask);

        println!("z {} children {} total_children {} {:?}", z, children, total_children, tile);

        level_tile_count += 1;
        // If we are done with the level, decrement z to the next zoom.
        if children >= total_children  && z > 0 {
            total_children = level_tile_count;
            level_tile_count = 0;
            children = 0;
            z -= 2;
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
