use super::{
    hilbert_tile::{Chunk, HilbertTile},
    leaf::Leaf,
};
use crate::{
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{Node, Relation, Way, Osm},
};
use std::{ops::Range, path::Path, fs::File};
use std::io::BufWriter;

pub fn build_chunks(
    m_leaves: &Mutant<Leaf>,
    m_tiles: &Mutant<HilbertTile>,
    dir: &Path,
    archive: &Osm,
    leaf_zoom: u8,
) -> Result<(Mutant<Chunk>, Mutant<Chunk>, Mutant<Chunk>), Box<dyn std::error::Error>> {
    let leaves = m_leaves.slice();
    let tiles = m_tiles.slice();
    let tiles_mut = m_tiles.mutable_slice();
    let nodes = archive.nodes();
    let ways = archive.ways();
    let relations = archive.relations();

    let n_chunks_writer = Mutant::<u64>::empty_buffered_writer(dir, "hilbert_n_chunks")?;
    let w_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_w_chunks")?;
    let r_chunks_writer = Mutant::<Chunk>::empty_buffered_writer(dir, "hilbert_r_chunks")?;
    let mut n_chunk_count = 0;
    let mut w_chunk_count = 0;
    let mut r_chunk_count = 0;

    let mut z = leaf_zoom - 2;
    let mut level_tile_count = 0;
    let mut total_children = leaves.len() as u32;
    let mut children = 0;
    for tile in tiles {
        children += count_children(tile.mask);

        println!(
            "z {} children {} total_children {} {:?}",
            z, children, total_children, tile
        );

        // Gather all of the children for the tile
        let children_indices = get_children_indices(tile.child, tile.mask);
        if z == leaf_zoom - 2 {
            for i in children_indices {
                let entity_ranges = get_entity_ranges(i, leaves, nodes, ways, relations);
                chunk_nodes(entity_ranges.0, nodes, &n_chunks_writer, &mut n_chunk_count, z);
                // chunk_ways(entity_ranges.1, ways, &w_chunks_file, &mut w_chunk_count);
                // chunk_relations(entity_ranges.2, relations, &r_chunks_file, &mut r_chunk_count);
            }
        } else {
            for i in children_indices {
                let tile = &tiles[i];
            }
        }

        level_tile_count += 1;
        // If we are done with the level, decrement z to the next zoom.
        if children >= total_children && z > 0 {
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

fn get_children_indices(first_child: u32, mask: u16) -> Vec<usize> {
    let mut indices = Vec::new();
    for i in 0..16 {
        if mask >> i & 1 == 1 {
            indices.push((first_child + i) as usize);
        }
    }
    indices
}

fn get_entity_ranges(
    i: usize,
    leaves: &[Leaf],
    nodes: &[Node],
    ways: &[Way],
    relations: &[Relation],
) -> (Range<usize>, Range<usize>, Range<usize>) {
    let leaf = &leaves[i];
    if i == leaves.len() - 1 {
        (
            (leaf.n as usize)..nodes.len(),
            (leaf.w as usize)..ways.len(),
            (leaf.r as usize)..relations.len(),
        )
    } else {
        let next_leaf = &leaves[i + 1];
        (
            (leaf.n as usize)..(next_leaf.n as usize),
            (leaf.w as usize)..(next_leaf.w as usize),
            (leaf.r as usize)..(next_leaf.r as usize),
        )
    }
}

fn chunk_nodes(range: Range<usize>, nodes: &[Node], writer: &BufWriter<File>, count: &mut u32, zoom: u8) {

    for i in range {
        let node = &nodes[i];
        
        *count += 1;
    }
    
}

#[cfg(test)]
mod tests {
    use flatdata::FileResourceStorage;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_chunks() {
        let dir = PathBuf::from("tests/fixtures/santacruz/sort");
        let archive = Osm::open(FileResourceStorage::new(&dir)).unwrap();
        let m_leaves = Mutant::<Leaf>::open(&dir, "hilbert_leaves", false).unwrap();
        let m_tiles = Mutant::<HilbertTile>::open(&dir, "hilbert_tiles", false).unwrap();
        let _ = build_chunks(&m_leaves, &m_tiles, &dir, &archive, 12);
    }
}
