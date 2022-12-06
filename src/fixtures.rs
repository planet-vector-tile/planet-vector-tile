mod filter;
mod hilbert;
mod location;
mod manifest;
mod mutant;
mod osmflat;
mod parallel;
pub mod pvt_builder;
mod rules;
mod sort;
mod source;
mod tile;
mod tile_attributes;

use fs_extra::dir::{copy, CopyOptions};
use hilbert::tree::HilbertTree;
use humantime::format_duration;
use std::{error::Error, fs, time::Instant};

fn main() {
    let time = Instant::now();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    let _ = fs::remove_dir_all("tests/fixtures/nodes4");
    let _ = fs::remove_dir_all("tests/fixtures/santacruz");

    build(
        "./tests/fixtures/nodes4_convert.toml",
        "tests/fixtures/nodes4_sort.toml",
    );
    build(
        "./tests/fixtures/santacruz_convert.toml",
        "tests/fixtures/santacruz_sort.toml",
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
    HilbertTree::build(sort_manifest).unwrap_or_else(quit);
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Fixture generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
