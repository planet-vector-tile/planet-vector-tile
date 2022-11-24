use std::path::PathBuf;

/// Compiler of Open Street Data from osm.pbf format to osm.flatdata format
#[derive(Debug, clap::Parser)]
#[clap(about, version, author)]
pub struct Args {
    /// Input OSM pbf file
    pub input: PathBuf,

    /// Output directory for osmflat
    pub output: PathBuf,

    /// Path to the manifest file (default: ./manifest.json)
    #[arg(short, long)]
    pub manifest: Option<PathBuf>,

    /// Include OSM entity ids.
    #[structopt(long = "ids")]
    pub ids: bool,

    /// Overwrite existing archive output directory
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}
