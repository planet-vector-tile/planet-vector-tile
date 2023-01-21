use flatdata::FileResourceStorage;
use gdal::Dataset;
use gdal::vector::{LayerAccess, OGRwkbGeometryType};
use itertools::Itertools;

use crate::manifest::Manifest;
use crate::osmflat::osmflat_generated::osm::{Osm, OsmBuilder};
use crate::util::{finish, timer};
use gdal::vector::FieldValue;

type Error = Box<dyn std::error::Error>;

pub fn convert(manifest: &Manifest) -> Result<Osm, Error> {
    let t = timer("Converting with gdal...");

    // Open source input file
    let source_dataset = match Dataset::open(manifest.data.source.as_path()) {
        Ok(f) => f,
        Err(err) => {
            eprintln!(
                "Unable to open source file: {}",
                manifest.data.source.display(),
            );
            eprintln!(
                "Are you pointing to the right source, planet, and archive in your manifest?"
            );
            return Err(Box::new(err));
        }
    };

    // NHTODO Handle projection for non-WGS84 inputs.

    // https://github.com/georust/gdal/blob/f463ba65592bead0675124702e5887d64710b489/src/vector/gdal_to_geo.rs

    for mut layer in source_dataset.layers() {
        let name = layer.name();
        for feature in layer.features() {
            let Some(geom) = feature.geometry() else { continue; };
            
            let geom_type = geom.geometry_type();

            match geom_type {
                OGRwkbGeometryType::wkbPoint => {
                    let (x, y, _) = geom.get_point(0);
                    
                }
                OGRwkbGeometryType::wkbMultiPoint => {
                    let point_count = geom.point_count();
                    for i in 0..point_count {
                        let (x, y, _) = geom.get_point(i);

                    }
                    let coords = (0..point_count)
                        .map(|n| {
                            unsafe { geo.get_unowned_geometry(n) }
                                .try_into()
                                .map(|inner_geom| match inner_geom {
                                    geo_types::Geometry::Point(p) => p,
                                    _ => panic!("Expected to get a Point"),
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(geo_types::Geometry::MultiPoint(geo_types::MultiPoint(
                        coords,
                    )))
                }
                OGRwkbGeometryType::wkbLineString => {
                    let coords = geo
                        .get_point_vec()
                        .iter()
                        .map(|&(x, y, _)| geo_types::Coord { x, y })
                        .collect();
                    Ok(geo_types::Geometry::LineString(geo_types::LineString(
                        coords,
                    )))
                }
                OGRwkbGeometryType::wkbMultiLineString => {
                    let string_count =
                        unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                    let strings = (0..string_count)
                        .map(|n| {
                            unsafe { geo.get_unowned_geometry(n) }
                                .try_into()
                                .map(|inner_geom| match inner_geom {
                                    geo_types::Geometry::LineString(s) => s,
                                    _ => panic!("Expected to get a LineString"),
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(geo_types::Geometry::MultiLineString(
                        geo_types::MultiLineString(strings),
                    ))
                }
                OGRwkbGeometryType::wkbPolygon => {
                    let ring_count =
                        unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                    let outer = ring(0)?;
                    let holes = (1..ring_count).map(ring).collect::<Result<Vec<_>, _>>()?;
                    Ok(geo_types::Geometry::Polygon(geo_types::Polygon::new(
                        outer, holes,
                    )))
                }
                OGRwkbGeometryType::wkbMultiPolygon => {
                    let string_count =
                        unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                    let strings = (0..string_count)
                        .map(|n| {
                            unsafe { geo.get_unowned_geometry(n) }
                                .try_into()
                                .map(|inner_geom| match inner_geom {
                                    geo_types::Geometry::Polygon(s) => s,
                                    _ => panic!("Expected to get a Polygon"),
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(
                        strings,
                    )))
                }
                OGRwkbGeometryType::wkbGeometryCollection => {
                    let item_count =
                        unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                    let geometry_list = (0..item_count)
                        .map(|n| unsafe { geo.get_unowned_geometry(n) }.try_into())
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(geo_types::Geometry::GeometryCollection(
                        geo_types::GeometryCollection(geometry_list),
                    ))
                }
                _ => Err(GdalError::UnsupportedGdalGeometryType(geometry_type)),
            }

            if let Some(id) = feature.fid() {

            }
            for (key, val) in feature.fields() {
                if let Some(fv) = val {
                    let str_val = match fv {
                        FieldValue::IntegerValue(v) => v.to_string(),
                        FieldValue::IntegerListValue(v) => v.iter().join(", "),
                        FieldValue::Integer64Value(v) => v.to_string(),
                        FieldValue::Integer64ListValue(v) => v.iter().join(", "),
                        FieldValue::StringValue(v) => v,
                        FieldValue::StringListValue(v) => v.iter().join(", "),
                        FieldValue::RealValue(v) => v.to_string(),
                        FieldValue::RealListValue(v) => v.iter().join(", "),
                        FieldValue::DateValue(v) => v.to_string(),
                        FieldValue::DateTimeValue(v) => v.to_string(),
                    };

                }
            }
            
        }
    }

    // Prepare flatdata
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
