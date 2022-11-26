use super::{
    chunk::build_chunks,
    hilbert_tile::{build_tiles, Chunk, HilbertTile},
    leaf::{build_leaves, populate_hilbert_leaves_external, Leaf},
};
use crate::{
    manifest::{self, Manifest},
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair, Osm}, tile::Tile,
};
use flatdata::FileResourceStorage;
use std::{fs, path::Path};

pub struct HilbertTree {
    pub manifest: Manifest,
    pub tiles: Mutant<HilbertTile>,
    pub leaves: Mutant<Leaf>,
    pub leaves_external: Mutant<u32>,
    pub n_chunks: Mutant<Chunk>,
    pub w_chunks: Mutant<Chunk>,
    pub r_chunks: Mutant<Chunk>,
    pub archive: Osm,
    pub way_pairs: Mutant<HilbertWayPair>,
}

impl HilbertTree {
    pub fn build(dir: &Path, manifest: Manifest) -> Result<Self, Box<dyn std::error::Error>> {
        let leaf_zoom = manifest.render.leaf_zoom;

        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, leaf_zoom)?;
        let m_tiles = build_tiles(&m_leaves, &dir, leaf_zoom)?;

        let archive = Osm::open(FileResourceStorage::new(dir))?;

        let manifest_str = toml::to_string(&manifest)?;
        fs::write(dir.join("manifest.toml"), manifest_str)?;

        let m_leaves_external = populate_hilbert_leaves_external(
            dir,
            &archive,
            &m_node_pairs,
            &m_way_pairs,
            &m_leaves,
            leaf_zoom,
        )?;

        let (n_chunks, w_chunks, r_chunks) = build_chunks(
            &m_leaves,
            &m_tiles,
            &m_leaves_external,
            &dir,
            &archive,
            &manifest,
        )?;

        Ok(Self {
            manifest,
            tiles: m_tiles,
            leaves: m_leaves,
            leaves_external: m_leaves_external,
            n_chunks,
            w_chunks,
            r_chunks,
            archive,
            way_pairs: m_way_pairs,
        })
    }

    pub fn open(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest = manifest::parse(Some(dir.join("manifest.toml")));
        let leaf_zoom = manifest.render.leaf_zoom;

        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;
        let m_leaves = Mutant::<Leaf>::open(dir, "hilbert_leaves", false)?;
        let m_tiles = Mutant::<HilbertTile>::open(dir, "hilbert_tiles", false)?;

        let n_chunks = Mutant::<Chunk>::new(dir, "hilbert_n_chunks", 1000)?;
        let w_chunks = Mutant::<Chunk>::new(dir, "hilbert_w_chunks", 1000)?;
        let r_chunks = Mutant::<Chunk>::new(dir, "hilbert_r_chunks", 1000)?;

        let archive = Osm::open(FileResourceStorage::new(dir))?;
        let m_leaves_external = populate_hilbert_leaves_external(
            dir,
            &archive,
            &m_node_pairs,
            &m_way_pairs,
            &m_leaves,
            leaf_zoom,
        )?;

        Ok(Self {
            manifest,
            tiles: m_tiles,
            leaves: m_leaves,
            leaves_external: m_leaves_external,
            n_chunks,
            w_chunks,
            r_chunks,
            archive,
            way_pairs: m_way_pairs,
        })
    }

    pub fn find(&self, tile: &Tile) -> FindResult {
        let leaf_zoom = self.manifest.render.leaf_zoom;

        // Tiles do not exist beyond the leaf zoom, and we only use even zoom levels.
        if tile.z & 1 == 1 || tile.z > leaf_zoom {
            return FindResult::None;
        }

        let h_tiles = self.tiles.slice();
        let leaves = self.leaves.slice();
        let mut h_tile = h_tiles.last().unwrap();
        let mut z = 2;
        let mut i = 0;
        while z <= tile.z {
            let h = tile.h >> (2 * (tile.z - z));
            i = match child_index(h_tile, h) {
                Some(i) => i,
                None => return FindResult::None,
            };
            // If we are all the way down to the leaves,
            // return a leaf result pair.
            if z == leaf_zoom {
                return FindResult::Leaf(ResultPair {
                    item: &leaves[i],
                    next: if i + 1 < leaves.len() {
                        Some(&leaves[i + 1])
                    } else {
                        None
                    },
                });
            }
            h_tile = &h_tiles[i];
            z += 2;
        }

        FindResult::HilbertTile(ResultPair {
            item: h_tile,
            next: if i + 1 < h_tiles.len() {
                Some(&h_tiles[i + 1])
            } else {
                None
            },
        })
    }
}

pub struct ResultPair<T> {
    pub item: T,
    pub next: Option<T>,
}

pub enum FindResult<'a> {
    HilbertTile(ResultPair<&'a HilbertTile>),
    Leaf(ResultPair<&'a Leaf>),
    None,
}

fn child_index(h_tile: &HilbertTile, child_h: u64) -> Option<usize> {
    let child_pos = child_h & 0xf;
    let mask = h_tile.mask;
    if mask >> child_pos & 1 != 1 {
        return None;
    }
    let mut offset = 0;
    for i in 0..child_pos {
        offset += mask >> i & 1;
    }
    Some(h_tile.child as usize + offset as usize)
}
