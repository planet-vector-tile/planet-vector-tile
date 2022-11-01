// This module's code is adapted from:
// https://github.com/boxdot/osmflat-rs

mod convert;
pub use convert::convert;

#[path = "../generated/osmflat_generated.rs"]
mod osmflat_generated;

#[path = "../generated/osmpbf.rs"]
mod osmpbf_generated;

mod ids;
mod osmpbf;
mod stats;
mod strings;
mod tags;
