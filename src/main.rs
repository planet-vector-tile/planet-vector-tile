mod args;
mod hilbert;
mod mutant;
mod osmflat;
mod parallel;
mod sort_archive;
mod tile;
mod location;

use args::*;
use clap::Parser;
use humantime::format_duration;
use std::{fs, time::Instant, error::Error};

fn main() {
    let time = Instant::now();

    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    if args.overwrite {
        if let Err(e) = fs::remove_dir_all(args.output.clone()) {
            eprintln!("Unable to remove output dir: {}", e);
        }
    }

    let dir = args.output.clone();

    let archive = osmflat::convert(&args).unwrap_or_else(quit);
    sort_archive::sort(archive, &dir).unwrap_or_else(quit);
    hilbert::HilbertTree::build(&dir, args.leafzoom).unwrap_or_else(quit);

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
