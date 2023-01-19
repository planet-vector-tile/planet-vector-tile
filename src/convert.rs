use crate::gdal;
use crate::osmflat::osmflat_generated::osm::Osm;
use crate::{manifest::Manifest, osmflat};
use std::io::ErrorKind;

type Error = Box<dyn std::error::Error>;

pub fn convert(manifest: &Manifest) -> Result<Osm, Error> {
    let Some(ext) = manifest.data.source.extension() else {
        let msg = format!("Source file {} in manifest has no extension.", manifest.data.source.as_path().display());
        let err = std::io::Error::new(ErrorKind::Other, msg);
        return Err(Box::new(err))
    };

    if ext == "pbf" {
        let flatdata = osmflat::convert(&manifest)?;
        Ok(flatdata)
    } else {
        let flatdata = gdal::convert(&manifest)?;
        Ok(flatdata)
    }
}
