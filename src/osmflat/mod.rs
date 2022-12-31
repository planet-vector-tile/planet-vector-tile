// This module's code is adapted from:
// https://github.com/boxdot/osmflat-rs

mod convert;
pub use convert::convert;

#[allow(dead_code)]
#[path = "../generated/osmflat_generated.rs"]
pub mod osmflat_generated;

#[path = "../generated/osmpbf.rs"]
mod osmpbf_generated;

pub mod ids;
pub mod osmpbf;
pub mod stats;
pub mod strings;
pub mod tags;
