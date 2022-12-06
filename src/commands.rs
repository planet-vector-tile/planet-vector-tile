use clap::{arg, Command};

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git/index.html

pub fn cli() -> Command {
    let pvt = Command::new("pvt")
        .about("Utility for generating Planet Vector Tiles")
        .subcommand_required(true)
        .arg_required_else_help(true);

    let manifest_arg = arg!(-m --manifest <MANIFEST_PATH> "Path to manifest file (default: ./manifest.json)")
        .default_value("./manifest.toml");

    let overwrite_arg = arg!(-o --overwrite "Overwrite existing planet data directory")
        .default_value("false")
        .value_name("OVERWRITE");

    let convert = Command::new("convert")
        .about("Converts an OSM PBF to planet data")
        .args([manifest_arg.clone(), overwrite_arg.clone()]);

    let render = Command::new("render")
        .about("Renders a Hilbert tile tree according to the layer rules in the manifest")
        .args([manifest_arg.clone(), overwrite_arg.clone()]);

    let archive = Command::new("archive")
        .about("Archives a planet data directory into a single PVT file")
        .args([manifest_arg.clone(), overwrite_arg.clone()]);

    let build = Command::new("build")
        .about("Converts, renders, and archives a planet")
        .args([manifest_arg.clone(), overwrite_arg.clone()]);

    pvt.subcommands([convert, render, archive, build])

}
