mod commands;
mod ingest;
mod filter;
mod hilbert;
mod location;
pub mod manifest;
mod mutant;
mod osmflat;
mod parallel;
mod planet;
pub mod pvt_builder;
mod pvt_yaml;
mod relation;
mod report;
mod rules;
mod sort;
mod source;
mod tile;
mod tile_attributes;
mod util;

use clap::ArgMatches;
use hilbert::tree::HilbertTree;
use humantime::format_duration;
use manifest::Manifest;
use std::{error::Error, fs};

fn main() {
    let time = util::timer("pvt");
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
            let manifest = get_manifest(matches);
            let overwrite = matches.get_one::<bool>("overwrite").unwrap();
            if *overwrite {
                if let Err(e) = fs::remove_dir_all(&manifest.data.planet) {
                    eprintln!("Unable to remove planet dir: {}", e);
                }
            }

            let flatdata = osmflat::convert(&manifest).unwrap_or_else(quit);
            sort::sort_flatdata(flatdata, &manifest.data.planet).unwrap_or_else(quit);

            match HilbertTree::new(&manifest) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!(
                        "Unable to open planet dir: {} Error: {:?}",
                        manifest.data.planet.display(),
                        e
                    );
                    eprintln!("Are you pointing to the right source, planet, and archive in your manifest?");
                    std::process::exit(1);
                }
            };
        }
        ("render", matches) => {
            let manifest = get_manifest(matches);

            let mut tree = match HilbertTree::open(&manifest) {
                Ok(tree) => tree,
                Err(e) => {
                    eprintln!(
                        "Unable to open planet dir: {} Error: {:?}",
                        manifest.data.planet.display(),
                        e
                    );
                    eprintln!("Are you pointing to the right source, planet, and archive in your manifest?");
                    std::process::exit(1);
                }
            };

            tree.render_tile_content().unwrap_or_else(quit);
        }
        ("archive", _) => {
            println!("TODO: Make a .pvt archive.")
        }
        ("build", matches) => {
            let manifest = get_manifest(matches);
            let overwrite = matches.get_one::<bool>("overwrite").unwrap();
            if *overwrite {
                if let Err(e) = fs::remove_dir_all(&manifest.data.planet) {
                    eprintln!("Unable to remove planet dir: {}", e);
                }
            }

            let flatdata = osmflat::convert(&manifest).unwrap_or_else(quit);
            sort::sort_flatdata(flatdata, &manifest.data.planet).unwrap_or_else(quit);

            let mut tree = match HilbertTree::new(&manifest) {
                Ok(tree) => tree,
                Err(e) => {
                    eprintln!(
                        "Unable to open planet dir: {} Error: {:?}",
                        manifest.data.planet.display(),
                        e
                    );
                    eprintln!("Are you pointing to the right source, planet, and archive in your manifest?");
                    std::process::exit(1);
                }
            };

            tree.render_tile_content().unwrap_or_else(quit);
        }
        ("report", matches) => {
            let manifest = get_manifest(matches);
            report::generate(&manifest).unwrap_or_else(quit);
        }
        _ => unreachable!(),
    }

    println!("Total Time: {}", format_duration(time.elapsed()));
}

fn quit<T>(e: Box<dyn Error>) -> T {
    eprintln!("Error: {}", e);
    std::process::exit(1);
}

fn get_manifest(matches: &ArgMatches) -> Manifest {
    let manifest_path_str = matches.get_one::<String>("MANIFEST_PATH").unwrap();

    let manifest = match manifest::parse(manifest_path_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };
    manifest
}
