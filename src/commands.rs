use clap::{arg, Command};

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git/index.html

pub fn cli() -> Command {
    let pvt = Command::new("pvt")
        .about("Utility for generating Planet Vector Tiles")
        .subcommand_required(true)
        .arg_required_else_help(true);

    let manifest_path = arg!(<MANIFEST_PATH> "Path to manifest file");

    let overwrite_arg = arg!(-o --overwrite "Overwrite existing planet")
        .default_value("false");

    let convert = Command::new("convert")
        .about("Converts an OSM PBF to planet data")
        .args([manifest_path.clone(), overwrite_arg.clone()]);

    let render = Command::new("render")
        .about("Renders a Hilbert tile tree according to the layer rules in the manifest")
        .args([manifest_path.clone(), overwrite_arg.clone()]);

    let archive = Command::new("archive")
        .about("Archives a planet data directory into a single PVT file")
        .args([manifest_path.clone(), overwrite_arg.clone()]);

    let build = Command::new("build")
        .about("Converts, renders, and archives a planet")
        .args([manifest_path.clone(), overwrite_arg.clone()]);

    pvt.subcommands([convert, render, archive, build])

}
