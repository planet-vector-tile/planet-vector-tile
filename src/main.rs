mod commands;
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

use clap::ArgMatches;
use hilbert::tree::HilbertTree;
use humantime::format_duration;
use manifest::Manifest;
use std::{error::Error, fs, time::Instant};

fn main() {
    let time = Instant::now();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    let matches = commands::cli().get_matches();

    let sub = match matches.subcommand() {
        Some(sub) => sub,
        None => {
            eprintln!(
                "pvt requires one of the following subcommands: convert, render, archive, build"
            );
            std::process::exit(1);
        }
    };

    match sub {
        ("convert", matches) => {
            let manifest = handle_args(matches);
            let archive = osmflat::convert(&manifest).unwrap_or_else(quit);
            sort::sort_flatdata(archive, &manifest.data.dir).unwrap_or_else(quit);
        }
        ("render", matches) => {
            let manifest = handle_args(matches);
            HilbertTree::build(manifest).unwrap_or_else(quit);

        }
        ("archive", _) => {
            println!("Make a .pvt archive.")
        }
        ("build", matches) => {
            let manifest = handle_args(matches);
            let archive = osmflat::convert(&manifest).unwrap_or_else(quit);
            sort::sort_flatdata(archive, &manifest.data.dir).unwrap_or_else(quit);
            HilbertTree::build(manifest).unwrap_or_else(quit);
        }
        _ => unreachable!(),
    }

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Planet generation FAILED!");
    eprintln!("Error: {}", e);
    std::process::exit(1);
}

fn handle_args(matches: &ArgMatches) -> Manifest {
    let manifest_str = matches.get_one::<String>("manifest").unwrap();
    let manifest = manifest::parse(Some(manifest_str.into()));

    let overwrite = matches.get_one::<bool>("overwrite").unwrap();

    if *overwrite {
        if let Err(e) = fs::remove_dir_all(&manifest.data.dir) {
            eprintln!("Unable to remove output dir: {}", e);
        }
    }

    manifest
}
