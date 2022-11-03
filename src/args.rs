use std::path::PathBuf;

/// Compiler of Open Street Data from osm.pbf format to osm.flatdata format
#[derive(Debug, clap::Parser)]
#[clap(about, version, author)]
pub struct Args {
    /// Input OSM pbf file
    pub input: PathBuf,

    /// Output directory for osmflat
    pub output: PathBuf,

    /// Include OSM entity ids.
    #[structopt(long = "ids")]
    pub ids: bool,

    /// Overwrite existing osmflat output directory
    #[arg(short, long, default_value_t = false)]
    pub overwrite: bool,

    /// Highest zoom at which tiles are indexed.
    #[arg(short, long, default_value_t = 12)]
    pub leafzoom: u8,
}
