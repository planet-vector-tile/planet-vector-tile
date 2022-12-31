use std::fs::File;
use std::time::Instant;

use crate::location;
use crate::osmflat::ids;
use crate::osmflat::osmpbf;
use crate::osmflat::osmpbf::read_block;
use crate::osmflat::osmpbf::BlockIndex;
use crate::osmflat::osmpbf::{build_block_index, BlockType};
use crate::osmflat::stats::Stats;
use crate::osmflat::strings::StringTable;
use crate::parallel;
use crate::planet::hilbert_pair::{HilbertPair32, HilbertPair40};
use crate::planet::node::Node;
use crate::planet::planet::Planet;
use crate::planet::tag::TagSerializer;
use crate::planet::EntityType;
use crate::u40::U40;
use itertools::Itertools;
use memmap2::Mmap;
use pbr::ProgressBar;
use std::str;

use crate::manifest::Manifest;
use crate::util::{finish, timer};

type Error = Box<dyn std::error::Error>;

pub fn ingest_osm_pbf(manifest: &Manifest) -> Result<Planet, Error> {
    let ingest_time = timer(&format!(
        "Ingesting {} into planet at {}",
        manifest.data.source.display(),
        manifest.data.planet.display()
    ));

    // Setup OSM PBF as memory mapped buffer.
    let pbf_file = match File::open(&manifest.data.source) {
        Ok(f) => f,
        Err(err) => {
            eprintln!(
                "Unable to open source file: {}",
                manifest.data.source.display(),
            );
            eprintln!(
                "Are you pointing to the right source, planet, and archive in your manifest?"
            );
            return Err(Box::new(err));
        }
    };
    let pbf: Mmap = unsafe { Mmap::map(&pbf_file)? };

    // Build an index of the PBF blocks so we know which block is which.
    let t = timer("Building index of PBF blocks.");
    let block_index = build_block_index(&pbf);
    finish(t);

    // Group blocks by type
    let blocks = group_blocks_by_type(block_index);

    // Initialize planet
    let mut planet = Planet::new(manifest)?;

    let mut stringtable = StringTable::new();
    let mut tags = TagSerializer::new(&planet)?;
    let mut stats = Stats::default();

    // NHTODO Ingest header

    // Ingest nodes
    let mut nodes_id_to_idx = ids::IdTableBuilder::new();
    let msg = "Ingesting nodes...";
    read_blocks(msg, &pbf, &blocks.nodes, &mut |block| {
        let string_refs = add_string_table(&block.stringtable, &mut stringtable)?;
        for group in block.primitivegroup.iter() {
            let dense_nodes = group.dense.as_ref().unwrap();

            let lat_offset = block.lat_offset.unwrap_or(0);
            let lon_offset = block.lon_offset.unwrap_or(0);
            let mut lat = 0;
            let mut lon = 0;

            let mut tags_offset = 0;

            let mut id = 0;
            for i in 0..dense_nodes.id.len() {
                id += dense_nodes.id[i];

                let index = nodes_id_to_idx.insert(id as u64);

                lat += dense_nodes.lat[i];
                lon += dense_nodes.lon[i];
                let lat_dm7 = ((lat_offset + lat) / 10_000_000 as i64) as i32;
                let lon_dm7 = ((lon_offset + lon) / 10_000_000 as i64) as i32;
                let h = location::lonlat_to_h((lon_dm7, lat_dm7));
                let hilbert_pair = HilbertPair40::new(h, i as u64);
                planet.node_pairs.unwrap().borrow_mut().push(hilbert_pair);

                // NHTODO Handle negative IDs.
                let pos_id = if id < 0 {
                    eprintln!("Negative node ID found. Net yet supported. Flipping to positive.");
                    -id as u64
                } else {
                    id as u64
                };

                let node = Node {
                    h,
                    id: U40::from_u64(pos_id),
                    lon: lon_dm7,
                    lat: lat_dm7,
                    tag_i: planet.node_tag_i.borrow().len as u32,
                };

                if tags_offset < dense_nodes.keys_vals.len() {
                    loop {
                        let k = dense_nodes.keys_vals[tags_offset];
                        tags_offset += 1;

                        if k == 0 {
                            break; // separator
                        }

                        let v = dense_nodes.keys_vals[tags_offset];
                        tags_offset += 1;

                        tags.serialize(
                            EntityType::Node,
                            string_refs[k as usize] as u32,
                            string_refs[v as usize] as u32,
                        )?;
                    }
                }
            }
            assert_eq!(tags_offset, dense_nodes.keys_vals.len());
            stats.num_nodes += dense_nodes.id.len();
        }

        Ok(())
    })?;

    // Ingest ways
    let msg = "Ingesting ways...";
    read_blocks(msg, &pbf, &blocks.ways, &mut |block| {
        println!("hi");
        Ok(())
    })?;

    // Ingest relations
    let msg = "Ingesting relations...";
    read_blocks(msg, &pbf, &blocks.relations, &mut |block| {
        println!("hi");
        Ok(())
    })?;

    let planet = Planet::new(manifest)?;

    finish(ingest_time);
    Ok(planet)
}

fn read_blocks(
    msg: &str,
    pbf: &Mmap,
    blocks: &[BlockIndex],
    cb: &mut dyn for<'r> FnMut(&'r osmpbf::PrimitiveBlock) -> Result<(), Error>,
) -> Result<(), Error> {
    let t = Instant::now();
    let mut pb = ProgressBar::new(blocks.len() as u64);
    pb.message(msg);

    parallel::parallel_process(
        blocks.into_iter(),
        |idx| read_block(pbf, &idx),
        |block| -> Result<osmpbf::PrimitiveBlock, Error> {
            let block = block?;

            cb(&block)?;

            pb.inc();
            Ok(block)
        },
    )?;

    finish(t);
    Ok(())
}

fn group_blocks_by_type(block_index: Vec<BlockIndex>) -> BlocksByType {
    let groups = block_index.into_iter().group_by(|b| b.block_type);
    let mut blocks_by_type = BlocksByType {
        header: Vec::new(),
        nodes: Vec::new(),
        ways: Vec::new(),
        relations: Vec::new(),
    };
    for (block_type, blocks) in &groups {
        match block_type {
            BlockType::Header => blocks_by_type.header = blocks.collect(),
            BlockType::Nodes => panic!("Found nodes block, only dense nodes are supported now"),
            BlockType::DenseNodes => blocks_by_type.nodes = blocks.collect(),
            BlockType::Ways => blocks_by_type.ways = blocks.collect(),
            BlockType::Relations => blocks_by_type.relations = blocks.collect(),
        }
    }
    blocks_by_type
}

struct BlocksByType {
    header: Vec<BlockIndex>,
    nodes: Vec<BlockIndex>,
    ways: Vec<BlockIndex>,
    relations: Vec<BlockIndex>,
}

/// adds all strings in a table to the lookup and returns a vectors of
/// references to be used instead
fn add_string_table(
    pbf_stringtable: &osmpbf::StringTable,
    stringtable: &mut StringTable,
) -> Result<Vec<u64>, Error> {
    let mut result = Vec::with_capacity(pbf_stringtable.s.len());
    for x in &pbf_stringtable.s {
        let string = str::from_utf8(x)?;
        result.push(stringtable.insert(string));
    }
    Ok(result)
}
