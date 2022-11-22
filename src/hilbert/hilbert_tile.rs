use std::path::Path;

use crate::mutant::Mutant;

use super::leaf::Leaf;

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
    // Index of the first child.
    pub child: u32,
    // Bit mask denoting which of the 16 children for the given tile exist.
    // MSB is index 15, MSB is index 0.
    pub mask: u16,
    // Indices to the first chunk of nodes, ways, relations from other tiles
    // that enter into this tile.
    n_chunk: u32,
    w_chunk: u32,
    r_chunk: u32,
}

// Chunks are offsets and run lengths of entities used for a given tile in the entity.
#[derive(Debug)]
pub struct Chunk {
    pub offset: u32, // offset from the w or r of the leaf tile
    pub len: u32,
}


pub fn build_tiles(
    m_leaves: &Mutant<Leaf>,
    dir: &Path,
    leaf_zoom: u8,
) -> Result<Mutant<HilbertTile>, Box<dyn std::error::Error>> {
    let leaves = m_leaves.slice();
    let max_tiles_len = max_tiles_len(&leaves, leaf_zoom);
    let mut m_tiles = Mutant::<HilbertTile>::new(dir, "hilbert_tiles", max_tiles_len)?;
    let tiles = m_tiles.mutable_slice();

    println!("TILES");

    // We only use even zooms.
    // Begin building at the last zoom before leaves
    let mut zoom = leaf_zoom - 2;
    // The index of the tile we are building
    let mut tiles_i = 0;
    // The index range of children for the level we are working on.
    let mut level_child_range = 0..leaves.len();

    // The last level (leaf_zoom - 2) has leaves for children, so we need to track this
    // to know when to look in the leaves array for chilrden.
    // 0 means we are still working on the leaves' parent level.
    let mut leaf_parent_level_end = 0;

    loop {
        println!(
            "<== ZOOM {} ==> start {} end {}",
            zoom, level_child_range.start, level_child_range.end
        );

        // The index to the first child tile we are looking at.
        let mut child_i = level_child_range.start;

        // Creating each of the tiles for a given level.
        while child_i < level_child_range.end {
            // Determine the end of the range of valid h for the tile we are building.
            let leaf_h = get_leaf_h(tiles, leaves, leaf_parent_level_end, child_i);
            let tile_h = leaf_to_tile_h(leaf_h, leaf_zoom, zoom);
            let h_range_end = child_h_range_end(tile_h);

            // The first child for the tile
            let first_child_i = child_i as u32;
            let leaf_h = get_leaf_h(tiles, leaves, leaf_parent_level_end, child_i);
            let mut child_h = leaf_to_tile_h(leaf_h, leaf_zoom, zoom + 2);

            // The mask we are building.
            let mut mask: u16 = 0;

            // Loop through the children to set the positional bits of the mask.
            loop {
                // Position the child is in of the possible children.
                let child_pos = (child_h & 0xf) as u16;
                let child_bit = 1 << child_pos;
                mask |= child_bit;
                child_i += 1;
                // Finished with children if at the end of level's child index range.
                if child_i == level_child_range.end {
                    break;
                }
                // Finished with children if the next child is not in the h range
                // of the tile we are building.
                let leaf_h = get_leaf_h(tiles, leaves, leaf_parent_level_end, child_i);
                child_h = leaf_to_tile_h(leaf_h, leaf_zoom, zoom + 2);
                if child_h >= h_range_end {
                    break;
                }
            }

            let tile = HilbertTile {
                child: first_child_i,
                mask,
                n_chunk: 0,
                w_chunk: 0,
                r_chunk: 0,
            };
            println!(
                "i {} z {} h {} mask {:#018b} {:?}",
                tiles_i, zoom, tile_h, tile.mask, tile
            );
            tiles[tiles_i] = tile;
            tiles_i += 1;
        }

        if zoom == leaf_zoom - 2 {
            level_child_range = 0..tiles_i;
            leaf_parent_level_end = tiles_i;
        } else {
            level_child_range = level_child_range.end..tiles_i;
        }

        // We are done if we just completed z0
        if zoom == 0 {
            break;
        }
        // The next tree level is two zoom levels down.
        zoom -= 2;
    }

    m_tiles.set_len(tiles_i);
    m_tiles.trim();

    Ok(m_tiles)
}

fn max_tiles_len(leaves: &[Leaf], leaf_zoom: u8) -> usize {
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

fn get_leaf_h(
    tiles: &[HilbertTile],
    leaves: &[Leaf],
    leaf_parent_level_end: usize,
    child_i: usize,
) -> u32 {
    // When still working on parent level of the leaves, the end is set to 0.
    if leaf_parent_level_end == 0 {
        return leaves[child_i].h;
    }

    let mut i = child_i;
    let mut tile = &tiles[i];
    while i > leaf_parent_level_end {
        let child = tile.child as usize;
        tile = &tiles[child];
        i = child;
    }
    // Now we are at the parent of leaf level
    tile = &tiles[i];
    i = tile.child as usize;
    leaves[i].h
}

fn leaf_to_tile_h(h: u32, leaf_zoom: u8, zoom: u8) -> u32 {
    h >> (2 * (leaf_zoom - zoom))
}

fn child_h_range_end(h: u32) -> u32 {
    let start = h << 4;
    start + 16
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

}