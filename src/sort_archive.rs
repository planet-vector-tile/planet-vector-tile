use std::{io::{Error, ErrorKind}, path::PathBuf};

use crate::mutant::{mutable_slice, Mutant};
use crate::osmflat::osmflat_generated::osm::{Osm, HilbertWayPair};

pub fn sort(archive: Osm, dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> { 
    let hilbert_node_pairs = match archive.hilbert_node_pairs() {
        Some(p) => p,
        None => {
            return Err(Box::new(Error::new(
                ErrorKind::NotFound,
                "No hilbert node pairs!",
            )));
        },
    };

    let node_pairs = mutable_slice(hilbert_node_pairs);

    let way_pairs = Mutant::<HilbertWayPair>::new(&dir, "hilbert_way_pairs", 20)?;
 
    // hexdump -C hilbert_way_pairs
    for slc in way_pairs.mutable_slice() {
        slc.set_h(u64::MAX);
        slc.set_i(0xABC);
    }

    
    Ok(())
}

