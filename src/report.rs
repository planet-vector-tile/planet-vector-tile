use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use crate::source::Source;
use crate::tile::Tile;
use chrono::Local;

use crate::pvt_builder::PVTBuilder;
use crate::pvt_yaml::PVTYaml;
use crate::tile::planet_vector_tile_generated::root_as_pvttile;

use crate::hilbert::tree::HilbertTree;
use crate::manifest::Manifest;

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub struct ReportOptions {
    pub write_fb_tiles: bool,
    pub lookup_strings_and_values: bool,
    pub include_strings: bool,
    pub include_values: bool,
    pub include_layers: bool,
    pub include_features: bool,
    pub include_geometries: bool,
}

pub fn generate(manifest: &Manifest) -> Result<(), Error> {
    let options = parse_options(&manifest.report_options);
    let date = Local::now();
    let date_fmt = date.format("%Y-%m-%d_%H:%M:%S");
    let file_name = format!("report-{}.yaml", date_fmt);
    let report_path = manifest.data.planet.join(file_name);
    println!("Generating report at: {}", report_path.display());

    let tiles_dir = manifest.data.planet.join("tiles");
    if options.write_fb_tiles {
        let _ = fs::remove_dir_all(&tiles_dir);
        if let Err(e) = fs::create_dir(&tiles_dir) {
            eprintln!("Unable to create tiles dir: {}", e);
        }
    }

    let file = File::create(report_path)?;
    let mut buf_writer = BufWriter::with_capacity(1024 * 1024 * 32, file);

    let tree = HilbertTree::open(manifest)?;

    // Iterate everything
    if manifest.data.include_leaves.is_empty() {
        let leaf_it = tree.pvt_leaf_iterator();
        for (tile, buffer) in leaf_it {
            visit_tile(&tile, &buffer, &mut buf_writer, &tiles_dir, &options)?;
        }
    }
    // Just iterate the included leaves and their parents
    else {
        let mut leaves = manifest.data.include_leaves.clone();
        leaves.sort();
        leaves.dedup();

        for h in leaves {
            let tile = Tile::from_zh(manifest.render.leaf_zoom, h);
            let mut builder = PVTBuilder::new();
            tree.compose_tile(&tile, &mut builder);
            let buffer = builder.build();

            visit_tile(&tile, &buffer, &mut buf_writer, &tiles_dir, &options)?;
        }
    }

    buf_writer.flush()?;
    Ok(())
}

fn visit_tile(
    tile: &Tile,
    buffer: &Vec<u8>,
    buf_writer: &mut BufWriter<File>,
    tiles_dir: &Path,
    options: &ReportOptions,
) -> Result<(), Error> {
    if options.write_fb_tiles {
        let file_name = format!("{}_{}.pvt", tile.z, tile.h);
        let mut file = File::create(tiles_dir.join(file_name))?;
        file.write_all(&buffer)?;
    }

    let size = buffer.len();
    let pvt = root_as_pvttile(&buffer)?;

    let yaml_string = pvt.to_yaml_report(&tile, size, options.clone());
    buf_writer.write_all(yaml_string.as_bytes())?;

    Ok(())
}

fn parse_options(strs: &Vec<String>) -> ReportOptions {
    let mut options = ReportOptions {
        write_fb_tiles: false,
        lookup_strings_and_values: false,
        include_strings: false, // This is the array of string values in the strings array of the PVT flatbuffer
        include_values: false, // This is the array of values in the values array of the PVT flatbuffer.
        include_layers: false,
        include_features: false,
        include_geometries: false,
    };

    for s in strs {
        match s.as_str() {
            "write_fb_tiles" => options.write_fb_tiles = true,
            "lookup_strings_and_values" => options.lookup_strings_and_values = true,
            "include_strings" => options.include_strings = true,
            "include_values" => options.include_values = true,
            "include_layers" => options.include_layers = true,
            "include_features" => options.include_features = true,
            "include_geometries" => options.include_geometries = true,
            _ => eprintln!("Unknown report option: {}", s),
        }
    }
    println!("{:?}", options);
    options
}
