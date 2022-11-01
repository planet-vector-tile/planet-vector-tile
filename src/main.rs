mod osmflat;
mod args;
mod parallel;

use args::*;
use clap::Parser;
use log::info;
use std::{time::Instant, fs};
use humantime::format_duration;

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_level(false)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    if args.overwrite {
        if let Err(e) = fs::remove_dir_all(args.output.clone()) {
            eprintln!("Unable to remove output dir: {}", e);
            std::process::exit(1);
        }
    }

    let t = Instant::now();
    if let Err(e) = osmflat::convert(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    info!("Conversion from osm.pbf to osm.flatdata is complete. {:?}", format_duration(t.elapsed()));
}
