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
use clap::Parser;
use humantime::format_duration;
use std::{error::Error, fs, time::Instant};
use hilbert::tree::HilbertTree;

fn main() {
    let time = Instant::now();

    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    let manifest = manifest::parse(args.manifest.clone());

    if args.overwrite {
        if let Err(e) = fs::remove_dir_all(args.output.clone()) {
            eprintln!("Unable to remove output dir: {}", e);
        }
    }

    let archive = osmflat::convert(&args).unwrap_or_else(quit);
    sort_archive::sort(archive, &args.output).unwrap_or_else(quit);
    HilbertTree::build(&args.output, manifest).unwrap_or_else(quit);

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
