mod filter;
mod hilbert;
mod location;
mod manifest;
mod mutant;
mod osmflat;
mod parallel;
mod planet;
pub mod pvt_builder;
mod relation;
mod rules;
mod sort;
mod source;
mod tile;
mod tile_attributes;
mod u40;
mod util;

use fs_extra::dir::{copy, CopyOptions};
use hilbert::tree::HilbertTree;
use humantime::format_duration;
use std::{error::Error, fs, time::Instant};

fn main() {
    let time = Instant::now();

    let _ = fs::remove_dir_all("tests/fixtures/nodes4");
    let _ = fs::remove_dir_all("tests/fixtures/santa_cruz");

    build(
        "./tests/fixtures/nodes4_convert.yaml",
        "tests/fixtures/nodes4_sort.yaml",
    );
    build(
        "./tests/fixtures/santa_cruz_convert.yaml",
        "tests/fixtures/santa_cruz_sort.yaml",
    );

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn build(convert_manifest_path_str: &str, sort_manifest_path_str: &str) {
    let convert_manifest = match manifest::parse(convert_manifest_path_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!(
                "Unable to parse manifest at: {} Err: {:?}",
                convert_manifest_path_str, e
            );
            quit(Box::new(e))
        }
    };

    let flatdata = osmflat::convert(&convert_manifest).unwrap_or_else(quit);

    let sort_manifest = match manifest::parse(sort_manifest_path_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!(
                "Unable to parse manifest at: {} Err: {:?}",
                convert_manifest_path_str, e
            );
            quit(Box::new(e))
        }
    };

    fs::create_dir_all(&sort_manifest.data.planet).unwrap();

    let mut opts = CopyOptions::default();
    opts.content_only = true;
    copy(
        convert_manifest.data.planet,
        &sort_manifest.data.planet,
        &opts,
    )
    .unwrap();

    sort::sort_flatdata(flatdata, &sort_manifest.data.planet).unwrap_or_else(quit);
    let mut tree = HilbertTree::new(&sort_manifest).unwrap_or_else(quit);
    tree.render_tile_content().unwrap_or_else(quit);
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Fixture generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
