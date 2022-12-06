use super::{
    content::render_tile_content,
    hilbert_tile::{build_tiles, HilbertTile},
    leaf::{build_leaves, populate_hilbert_leaves_external, Leaf},
};
use crate::{
    manifest::Manifest,
    mutant::Mutant,
    osmflat::osmflat_generated::osm::{HilbertNodePair, HilbertWayPair, Osm},
    tile::Tile,
};
use flatdata::FileResourceStorage;
use std::fs;

type Err = Box<dyn std::error::Error>;

pub struct HilbertTree {
    pub manifest: Manifest,
    pub tiles: Mutant<HilbertTile>,
    pub leaves: Mutant<Leaf>,
    pub leaves_external: Mutant<u32>,
    pub n: Mutant<u64>,
    pub w: Mutant<u32>,
    pub r: Mutant<u32>,
    pub flatdata: Osm,
    pub way_pairs: Mutant<HilbertWayPair>,
}

impl HilbertTree {
    pub fn new(manifest: &Manifest) -> Result<Self, Err> {
        let dir = &manifest.data.planet.clone();

        // Copy the manifest to the build directory so we know exactly what it was at the time of build.
        let manifest_str = toml::to_string(&manifest)?;
        fs::write(dir.join("manifest.toml"), manifest_str)?;

        let leaf_zoom = manifest.render.leaf_zoom;
        let flatdata = Osm::open(FileResourceStorage::new(dir))?;

        let m_node_pairs = Mutant::<HilbertNodePair>::open(dir, "hilbert_node_pairs", true)?;
        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;

        let m_leaves = build_leaves(&m_node_pairs, &m_way_pairs, &dir, leaf_zoom)?;
        let m_tiles = build_tiles(&m_leaves, &dir, leaf_zoom)?;

        let m_leaves_external = populate_hilbert_leaves_external(
            dir,
            &flatdata,
            &m_node_pairs,
            &m_way_pairs,
            &m_leaves,
            leaf_zoom,
        )?;

        Ok(Self {
            manifest: manifest.clone(),
            tiles: m_tiles,
            leaves: m_leaves,
            leaves_external: m_leaves_external,
            n: Mutant::<u64>::new(&dir, "n", 0)?,
            w: Mutant::<u32>::new(&dir, "w", 0)?,
            r: Mutant::<u32>::new(&dir, "r", 0)?,
            flatdata,
            way_pairs: m_way_pairs,
        })
    }

    pub fn render_tile_content(&mut self) -> Result<&Self, Err> {
        let (n, w, r) = render_tile_content(
            &self.leaves,
            &self.tiles,
            &self.leaves_external,
            &self.flatdata,
            &self.manifest,
        )?;
        self.n = n;
        self.w = w;
        self.r = r;
        Ok(self)
    }

    pub fn open(manifest: &Manifest) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = &manifest.data.planet;
        let flatdata = Osm::open(FileResourceStorage::new(dir))?;

        let m_way_pairs = Mutant::<HilbertWayPair>::open(dir, "hilbert_way_pairs", true)?;
        let m_leaves = Mutant::<Leaf>::open(dir, "hilbert_leaves", false)?;
        let m_leaves_external = Mutant::<u32>::open(dir, "hilbert_leaves_external", false)?;
        let m_tiles = Mutant::<HilbertTile>::open(dir, "hilbert_tiles", false)?;
        let m_n = Mutant::<u64>::open(dir, "n", false)?;
        let m_w = Mutant::<u32>::open(dir, "w", false)?;
        let m_r = Mutant::<u32>::open(dir, "r", false)?;

        Ok(Self {
            manifest: manifest.clone(),
            tiles: m_tiles,
            leaves: m_leaves,
            leaves_external: m_leaves_external,
            n: m_n,
            w: m_w,
            r: m_r,
            flatdata,
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

#[cfg(test)]
mod tests {
    use crate::manifest;

    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_basic_find() {
        // Scotts Valley
        // z 12 x 659 y 1593
        let t = Tile::from_zh(12, 3329134);

        let manifest = manifest::parse("tests/fixtures/santacruz_sort.toml").unwrap();
        let tree = HilbertTree::open(&manifest).unwrap();

        match tree.find(&t) {
            FindResult::HilbertTile(_) => panic!("Should not be a HilbertTile. Should be a leaf"),
            FindResult::Leaf(pair) => {
                let leaf = pair.item;
                let n = leaf.n;
                let w = leaf.w;
                let r = leaf.r;
                let h = leaf.h;
                assert_eq!(n, 865693);
                assert_eq!(w, 98587);
                assert_eq!(r, 0);
                assert_eq!(h, 3329134);
            }
            FindResult::None => panic!("Should be a leaf."),
        }
    }

    #[test]
    fn test_struct_size() {
        assert_eq!(22, size_of::<HilbertTile>());
        assert_eq!(28, size_of::<Leaf>());
    }
}
