use std::fs::File;
use std::time::Instant;

use crate::osmflat::osmpbf;
use crate::osmflat::osmpbf::read_block;
use crate::osmflat::osmpbf::BlockIndex;
use crate::osmflat::osmpbf::{build_block_index, BlockType};
use crate::parallel;
use crate::planet::planet::Planet;
use itertools::Itertools;
use memmap2::Mmap;
use pbr::ProgressBar;

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

    // Build strings table
    channel_blocks("Building strings table...", &pbf, &block_index, &|block| {
        println!("hi");
    })?;

    // Group blocks by type
    let blocks = group_blocks_by_type(block_index);

    // Ingest header
    if let Some(header) = blocks.header.first() {
        // serialize_header(&pbf_header, coord_scale, &builder, &mut stringtable)?;
    } else {
        println!("Missing header block from OSM PBF.");
    }

    // Ingest nodes
    channel_blocks("Ingesting nodes...", &pbf, &blocks.nodes, &|block| {
        println!("hi");
    })?;

    // Ingest ways
    channel_blocks("Ingesting ways...", &pbf, &blocks.ways, &|block| {
        println!("hi");
    })?;

    // Ingest relations
    channel_blocks("Ingesting relations...", &pbf, &blocks.relations, &|block| {
        println!("hi");
    })?;

    

    let planet = Planet::new(manifest)?;

    finish(ingest_time);
    Ok(planet)
}

fn channel_blocks(
    msg: &str,
    pbf: &Mmap,
    blocks: &[BlockIndex],
    cb: &dyn Fn(&osmpbf::PrimitiveBlock) -> (),
) -> Result<(), Error> {
    let t = Instant::now();
    let mut pb = ProgressBar::new(blocks.len() as u64);
    pb.message(msg);

    parallel::parallel_process(
        blocks.into_iter(),
        |idx| read_block(pbf, &idx),
        |block| -> Result<osmpbf::PrimitiveBlock, Error> {
            let block = block?;

            cb(&block);

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
