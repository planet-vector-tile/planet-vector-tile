use crate::{
    hilbert::{HilbertTile, HilbertTree, Leaf},
    pvt_builder::PVTBuilder,
    source::Source,
    tile::{
        planet_vector_tile_generated::{PVTFeature, PVTFeatureArgs, PVTGeometry, PVTGeometryArgs, PVTLayer, PVTLayerArgs},
        Tile,
    },
    tile_attributes::TileAttributes,
};

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

        let next_h_tile = if i + 1 < h_tiles.len() {
            Some(&h_tiles[i + 1])
        } else {
            None
        };

        let mut leaf = None;
        let mut next_leaf = None;
        if h_tile.mask == 0 {
            let leaves = self.leaves.slice();
            leaf = Some(&leaves[i]);
            if i + 1 < leaves.len() {
                next_leaf = Some(&leaves[i + 1]);
            }
        }

        Some(Result {
            h_tile,
            next_h_tile: None,
            leaf,
            next_leaf: None,
        })
    }

    pub fn compose_tile(&self, tile: Tile) -> Vec<u8> {
        match self.find(tile) {
            Some(res) => {
                if let Some(leaf) = res.leaf {
                    print!("leaf found {:?}", leaf);

                    let nodes = self.archive.nodes();
                    let ways = self.archive.ways();
                    let relations = self.archive.relations();

                    let (n_end, w_end, r_end) = if let Some(next_leaf) = res.next_leaf {
                        (next_leaf.n, next_leaf.w, next_leaf.r)
                    } else {
                        (
                            nodes.len() as u64,
                            ways.len() as u32,
                            relations.len() as u32,
                        )
                    };
                }

                Vec::new()
            }
            None => Vec::new(),
        }
    }
}

impl Source for HilbertTree {
    fn build_tile(&self, tile: &Tile, builder: &mut PVTBuilder) {
        let fbb = &mut builder.fbb;
        let attributes = &mut builder.attributes;

        let hello_key = attributes.upsert_string("hello");
        let world_val = attributes.upsert_string("world");
        let hilbert = attributes.upsert_string("hilbert");

        // Create center geometry
        let center = tile.project(tile.center());
        let center_path = fbb.create_vector(&[center]);
        let center_geom = PVTGeometry::create(
            fbb,
            &PVTGeometryArgs {
                points: Some(center_path),
            },
        );

        let keys = fbb.create_vector(&[hello_key]);
        let vals = fbb.create_vector(&[world_val]);
        let center_geoms = fbb.create_vector(&[center_geom]);

        // Create center feature.
        let center_feature = PVTFeature::create(
            fbb,
            &PVTFeatureArgs {
                id: tile.h,
                h: tile.h,
                keys: Some(keys),
                values: Some(vals),
                geometries: Some(center_geoms),
            },
        );

        let center_features = fbb.create_vector(&[center_feature]);
        let center_layer = PVTLayer::create(
            fbb,
            &PVTLayerArgs {
                name: hilbert,
                features: Some(center_features),
            },
        );

        builder.add_layer(center_layer);
        
    }
}
