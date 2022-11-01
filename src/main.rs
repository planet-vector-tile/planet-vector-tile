mod args;
mod osmflat;
mod parallel;
mod sort_archive;

use args::*;
use clap::Parser;
use humantime::format_duration;
use std::{fs, time::Instant};

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

    let time = Instant::now();

    let quit = |e| {
        eprintln!("Planet generation FAILED!");
        eprintln!("Error: {}", e);
        println!("Total Time: {:?}", format_duration(time.elapsed()));
        std::process::exit(1);
    };

    let archive = osmflat::convert(args).unwrap_or_else(|e| quit(e));
    sort_archive::sort(archive);

    println!("Total Time: {:?}", format_duration(time.elapsed()));
}
