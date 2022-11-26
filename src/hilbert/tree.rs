use super::{
    chunk::build_chunks,
    hilbert_tile::{build_tiles, Chunk, HilbertTile},
    leaf::{build_leaves, populate_hilbert_leaves_external, Leaf},
};
use crate::{
    manifest::{self, Manifest},
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair, Osm},
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
}
