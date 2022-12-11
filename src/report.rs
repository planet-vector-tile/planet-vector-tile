use std::io::BufWriter;
use std::io::Write;

use chrono::Local;
use crate::tile::Tile;
use crate::source::Source;

use crate::pvt_yaml::PVTYaml;
use crate::pvt_builder::PVTBuilder;
use crate::tile::planet_vector_tile_generated::root_as_pvttile;

use crate::hilbert::tree::HilbertTree;
use crate::manifest::Manifest;

#[derive(Debug, Clone)]
pub struct ReportOptions {
    pub lookup_strings_and_values: bool,
    pub include_strings: bool,
    pub include_values: bool,
    pub include_layers: bool,
    pub include_features: bool,
    pub include_geometries: bool,
}

pub fn generate(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options(&manifest.report_options);
    let date = Local::now();
    let date_fmt = date.format("%Y-%m-%d_%H:%M:%S");
    let file_name = format!("report-{}.yaml", date_fmt);
    let report_path = manifest.data.planet.join(file_name);
    println!("Generating report at: {}", report_path.display());

    let file = std::fs::File::create(report_path)?;
    let mut buf_writer = BufWriter::with_capacity(1024 * 1024 * 32, file);

    let tree = HilbertTree::open(manifest)?;

    // Iterate everything
    if manifest.data.include_leaves.is_empty() {
        let leaf_it = tree.pvt_leaf_iterator();
        for (tile, buffer) in leaf_it {
            let size = buffer.len();

            let pvt = match root_as_pvttile(&buffer) {
                Ok(pvt) => pvt,
                Err(e) => {
                    eprintln!("{} Error: {:?}", tile, e);
                    continue;
                }
            };

            let yaml_string = pvt.to_yaml_report(&tile, size, options.clone());
            buf_writer.write_all(yaml_string.as_bytes())?;
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
            let size = buffer.len();

            let pvt = match root_as_pvttile(&buffer) {
                Ok(pvt) => pvt,
                Err(e) => {
                    eprintln!("{} Error: {:?}", tile, e);
                    continue;
                }
            };

            let yaml_string = pvt.to_yaml_report(&tile, size, options.clone());
            buf_writer.write_all(yaml_string.as_bytes())?;
        }
    }

    Ok(())
}

fn parse_options(strs: &Vec<String>) -> ReportOptions {
    let mut options = ReportOptions {
        lookup_strings_and_values: false,
        include_strings: false,
        include_values: false,
        include_layers: false,
        include_features: false,
        include_geometries: false,
    };

    for s in strs {
        match s.as_str() {
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
