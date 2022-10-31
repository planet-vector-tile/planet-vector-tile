mod osmflat_generated;
mod tags;

#[path = "../generated/planet_vector_tile_generated.rs"]
pub use crate::osmflat_generated::osm::*;

pub use crate::tags::*;

// re-export what is needed from flatdata to use osmflat
pub use flatdata::FileResourceStorage;
#[cfg(feature = "tar")]
pub use flatdata::TarArchiveResourceStorage;
