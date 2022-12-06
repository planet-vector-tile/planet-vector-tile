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
use std::{error::Error, fs, path::PathBuf, time::Instant};

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
        "./tests/fixtures/fixtures/nodes4.toml",
        "tests/fixtures/nodes4/santacruz/sort",
    );
    build(
        "./tests/fixtures/santacruz.toml",
        "tests/fixtures/santacruz/sort",
    );

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn build(manifest_path_str: &str, sort_dir_str: &str) {
    let manifest = match manifest::parse(manifest_path_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!(
                "Unable to parse manifest at: {} Err: {:?}",
                manifest_path_str, e
            );
            quit(Box::new(e))
        }
    };

    let convert_dir = manifest.data.planet.clone();
    let flatdata = osmflat::convert(&manifest).unwrap_or_else(quit);

    let sort_dir = PathBuf::from(sort_dir_str);
    fs::create_dir_all(&sort_dir).unwrap();

    let mut opts = CopyOptions::default();
    opts.content_only = true;
    copy(convert_dir, &sort_dir, &opts).unwrap();

    let mut sort_manifest = manifest.clone();
    sort_manifest.data.planet = sort_dir.clone();

    sort::sort_flatdata(flatdata, &sort_dir).unwrap_or_else(quit);
    HilbertTree::build(sort_manifest).unwrap_or_else(quit);
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Fixture generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
