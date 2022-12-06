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

    let manifest1 = manifest::parse("./tests/fixtures/nodes4.toml");
    let manifest2 = manifest::parse("./tests/fixtures/santacruz.toml");
    let convert_dir1 = manifest1.data.planet.clone();
    let convert_dir2 = manifest2.data.planet.clone();

    let a1 = osmflat::convert(&manifest1).unwrap_or_else(quit);
    let a2 = osmflat::convert(&manifest2).unwrap_or_else(quit);

    let sort_dir1 = PathBuf::from("tests/fixtures/nodes4/sort");
    let sort_dir2 = PathBuf::from("tests/fixtures/santacruz/sort");

    fs::create_dir_all(&sort_dir1).unwrap();
    fs::create_dir_all(&sort_dir2).unwrap();

    let mut opts = CopyOptions::default();
    opts.content_only = true;
    copy(convert_dir1, &sort_dir1, &opts).unwrap();
    copy(convert_dir2, &sort_dir2, &opts).unwrap();

    let mut sort_manifest1 = manifest1.clone();
    sort_manifest1.data.planet = sort_dir1.clone();
    let mut sort_manifest2 = manifest2.clone();
    sort_manifest2.data.planet = sort_dir2.clone();

    sort::sort_flatdata(a1, &sort_dir1).unwrap_or_else(quit);
    HilbertTree::build(sort_manifest1).unwrap_or_else(quit);

    sort::sort_flatdata(a2, &sort_dir2).unwrap_or_else(quit);
    HilbertTree::build(sort_manifest2).unwrap_or_else(quit);

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Fixture generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
