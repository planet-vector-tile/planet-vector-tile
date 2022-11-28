use super::{
    content::populate_tile_content,
    hilbert_tile::{build_tiles, HilbertTile},
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
    pub n: Mutant<u64>,
    pub w: Mutant<u32>,
    pub r: Mutant<u32>,
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

        let (n, w, r) = populate_tile_content(
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
            n,
            w,
            r,
            archive,
            way_pairs: m_way_pairs,
        })
    }

    pub fn open(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest = manifest::parse(Some(dir.join("manifest.toml")));
        let archive = Osm::open(FileResourceStorage::new(dir))?;

        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;
        let m_leaves = Mutant::<Leaf>::open(dir, "hilbert_leaves", false)?;
        let m_leaves_external = Mutant::<u32>::open(dir, "hilbert_leaves_external", false)?;
        let m_tiles = Mutant::<HilbertTile>::open(dir, "hilbert_tiles", false)?;
        let m_n = Mutant::<u64>::open(dir, "n", false)?;
        let m_w = Mutant::<u32>::open(dir, "w", false)?;
        let m_r = Mutant::<u32>::open(dir, "r", false)?;

        Ok(Self {
            manifest,
            tiles: m_tiles,
            leaves: m_leaves,
            leaves_external: m_leaves_external,
            n: m_n,
            w: m_w,
            r: m_r,
            archive,
            way_pairs: m_way_pairs,
        })
    }
}
