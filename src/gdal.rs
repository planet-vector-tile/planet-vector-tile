use flatdata::FileResourceStorage;

use crate::manifest::Manifest;
use crate::osmflat::osmflat_generated::osm::{Osm, OsmBuilder};
use crate::util::{finish, timer};

type Error = Box<dyn std::error::Error>;

pub fn convert(manifest: &Manifest) -> Result<Osm, Error> {
    let t = timer("Converting with gdal...");

    let storage = FileResourceStorage::new(&manifest.data.planet);
    let builder = match OsmBuilder::new(storage.clone()) {
        Ok(builder) => builder,
        Err(e) => {
            eprintln!(
                "Unable to create new flatdata at {}",
                &manifest.data.planet.display()
            );
            eprintln!("If you want to overwrite an existing planet, add the argument --overwrite.");
            return Err(Box::new(e));
        }
    };

    std::mem::drop(builder);

    // NHTODO open real flatdata
    let storage = FileResourceStorage::new("/Users/n/code/planet-vector-tile/planets/santa_cruz");
    let flatdata = Osm::open(storage)?;

    finish(t);
    Ok(flatdata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest;

    #[test]
    fn convert_natural_earth_countries() {
        let manifest = manifest::parse("manifests/natural_earth_countries_convert.yaml").unwrap();
        let _ = convert(&manifest).unwrap();
    }
}
