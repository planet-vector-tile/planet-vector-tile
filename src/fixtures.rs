mod args;
mod manifest;
mod hilbert;
mod location;
mod mutant;
mod osmflat;
mod parallel;
pub mod pvt_builder;
mod sort_archive;
mod source;
mod tile;
mod tile_attributes;

use args::*;
use fs_extra::dir::{copy, CopyOptions};
use humantime::format_duration;
use std::{error::Error, fs, path::PathBuf, time::Instant};
use hilbert::tree::HilbertTree;

fn main() {
    let time = Instant::now();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    let _ = fs::remove_dir_all("tests/fixtures/nodes4");
    let _ = fs::remove_dir_all("tests/fixtures/santacruz");

    let args1 = Args {
        input: "tests/fixtures/nodes4.osm.pbf".into(),
        output: "tests/fixtures/nodes4/convert".into(),
        manifest: None,
        overwrite: true,
        ids: false,
    };
    let args2 = Args {
        input: "tests/fixtures/santacruz.osm.pbf".into(),
        output: "tests/fixtures/santacruz/convert".into(),
        manifest: None,
        overwrite: true,
        ids: false,
    };

    let a1 = osmflat::convert(&args1).unwrap_or_else(quit);
    let a2 = osmflat::convert(&args2).unwrap_or_else(quit);

    let dir1 = PathBuf::from("tests/fixtures/nodes4/sort");
    let dir2 = PathBuf::from("tests/fixtures/santacruz/sort");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    let mut opts = CopyOptions::default();
    opts.content_only = true;
    copy("./tests/fixtures/nodes4/convert", &dir1, &opts).unwrap();
    copy("./tests/fixtures/santacruz/convert", &dir2, &opts).unwrap();

    let manifest = manifest::parse(&None);

    sort_archive::sort(a1, &dir1).unwrap_or_else(quit);
    HilbertTree::build(&dir1, manifest.render.leaf_zoom).unwrap_or_else(quit);

    sort_archive::sort(a2, &dir2).unwrap_or_else(quit);
    HilbertTree::build(&dir2, manifest.render.leaf_zoom).unwrap_or_else(quit);

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Fixture generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
