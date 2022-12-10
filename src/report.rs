use std::{
    io::Error,
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::manifest::Manifest;
use crate::osmflat::osmflat_generated::osm::{Osm, Tag, TagIndex};
use flatdata::{FileResourceStorage, RawData};
use itertools::Itertools;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub fn generate(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = manifest.data.planet.join("report.yaml");
    println!("Generating report at: {}", report_path.display());

    Ok(())
}
