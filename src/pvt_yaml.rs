use crate::report::ReportOptions;
use crate::tile::planet_vector_tile_generated::{PVTTile,PVTValue, PVTValueType};
use crate::tile::Tile;

use yaml_rust::yaml;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlEmitter;

pub trait PVTYaml {
    fn to_yaml_report(&self, tile: &Tile, size: usize, options: ReportOptions) -> String;
}

impl PVTYaml for PVTTile<'_> {
    fn to_yaml_report(&self, tile: &Tile, size: usize, options: ReportOptions) -> String {
        let strings_lookup = StringsLookup::new(self.strings());
        let values_lookup = ValuesLookup::new(self.values(), &strings_lookup);

        let mut yaml_string = String::new();
        let mut emitter = YamlEmitter::new(&mut yaml_string);
        emitter.compact(true);
        let mut doc = yaml::Hash::with_capacity(3);

        doc.insert(
            Yaml::String("tile".to_string()),
            Yaml::String(tile.to_string()),
        );

        let size_str = if size > 1024 * 1024 {
            format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.2} KB", size as f64 / 1024.0)
        };
        doc.insert(Yaml::String("size".to_string()), Yaml::String(size_str));

        if let Some(layers) = self.layers() {
            let mut layers_arr = yaml::Array::with_capacity(layers.len());
            for layer in layers.iter() {
                let mut layer_hash = yaml::Hash::with_capacity(2);
                
                let name = if options.lookup_strings_and_values {
                    strings_lookup.get(layer.name())
                } else {
                    Yaml::Integer(layer.name() as i64)
                };
                layer_hash.insert(
                    Yaml::String("name".to_string()),
                    name,
                );

                if options.include_features {
                    if let Some(features) = layer.features() {
                        let mut features_arr = yaml::Array::with_capacity(features.len());
                        for feature in features.iter() {
                            let mut feature_hash = yaml::Hash::with_capacity(4);
                            feature_hash.insert(
                                Yaml::String("id".to_string()),
                                // Coercing to string because serializer doesnt support u64
                                Yaml::String(format!("{}", feature.id())),
                            );

                            if let Some(keys) = feature.keys() {
                                let mut keys_arr = yaml::Array::with_capacity(keys.len());
                                for key in keys.iter() {
                                    let k = if options.lookup_strings_and_values {
                                        strings_lookup.get(key)
                                    } else {
                                        Yaml::Integer(key as i64)
                                    };
                                    keys_arr.push(k);
                                }
                                feature_hash.insert(
                                    Yaml::String("keys".to_string()),
                                    Yaml::Array(keys_arr),
                                );
                            }
                            if let Some(values) = feature.values() {
                                let mut values_arr = yaml::Array::with_capacity(values.len());
                                for value in values.iter() {
                                    let v = if options.lookup_strings_and_values {
                                        values_lookup.get(value)
                                    } else {
                                        Yaml::Integer(value as i64)
                                    };
                                    values_arr.push(v);
                                }
                                feature_hash.insert(
                                    Yaml::String("values".to_string()),
                                    Yaml::Array(values_arr),
                                );
                            }

                            if options.include_geometries {
                                if let Some(geometries) = feature.geometries() {
                                    let mut geometries_arr =
                                        yaml::Array::with_capacity(geometries.len());
                                    for geometry in geometries.iter() {
                                        if let Some(points) = geometry.points() {
                                            let mut points_arr =
                                                yaml::Array::with_capacity(points.len());
                                            for point in points.iter() {
                                                let mut point_hash = yaml::Hash::with_capacity(2);
                                                point_hash.insert(
                                                    Yaml::String("x".to_string()),
                                                    Yaml::Integer(point.x() as i64),
                                                );
                                                point_hash.insert(
                                                    Yaml::String("y".to_string()),
                                                    Yaml::Integer(point.y() as i64),
                                                );
                                                points_arr.push(Yaml::Hash(point_hash));
                                            }
                                            geometries_arr.push(Yaml::Array(points_arr));
                                        }
                                    }
                                    feature_hash.insert(
                                        Yaml::String("geometries".to_string()),
                                        Yaml::Array(geometries_arr),
                                    );
                                }
                            }
                            features_arr.push(Yaml::Hash(feature_hash));
                        }
                        layer_hash.insert(
                            Yaml::String("features".to_string()),
                            Yaml::Array(features_arr),
                        );
                    }
                }
                layers_arr.push(Yaml::Hash(layer_hash));
            }
            doc.insert(Yaml::String("layers".to_string()), Yaml::Array(layers_arr));
        }

        if options.include_strings {
            if let Some(strings) = self.strings() {
                let mut strings_arr = yaml::Array::with_capacity(strings.len());
                for string in strings.iter() {
                    strings_arr.push(Yaml::String(string.to_string()));
                }
                doc.insert(
                    Yaml::String("strings".to_string()),
                    Yaml::Array(strings_arr),
                );
            }
        }

        if options.include_values {
            if let Some(values) = self.values() {
                let mut values_arr = yaml::Array::with_capacity(values.len());
                for value in values.iter() {
                    let t = value.t().variant_name().unwrap();
                    let v = value.v();
                    let mut hash = yaml::Hash::with_capacity(2);
                    hash.insert(Yaml::String("t".to_string()), Yaml::String(t.to_string()));
                    hash.insert(Yaml::String("v".to_string()), Yaml::Real(v.to_string()));
                    values_arr.push(Yaml::Hash(hash));
                }
                doc.insert(Yaml::String("values".to_string()), Yaml::Array(values_arr));
            }
        }

        match emitter.dump(&Yaml::Hash(doc)) {
            Ok(_) => {}
            Err(e) => {
                println!("Error writing YAML report for tile {} Err: {}", tile, e);
            }
        }

        yaml_string.push('\n');
        yaml_string
    }
}

struct StringsLookup<'a> {
    strings: Vec<&'a str>,
}

impl<'a> StringsLookup<'a> {
    pub fn new(
        fb_strings: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>>,
    ) -> Self {
        let strings = match fb_strings {
            Some(strings) => strings.iter().collect(),
            None => Vec::new(),
        };
        Self { strings }
    }

    pub fn get(&self, i: u32) -> Yaml {
        let s = self.strings[i as usize];
        Yaml::String(s.to_string())
    }
}

struct ValuesLookup<'a> {
    values: Vec<&'a PVTValue>,
    strings_lookup: &'a StringsLookup<'a>,
}

impl<'a> ValuesLookup<'a> {
    pub fn new(
        fb_values: Option<flatbuffers::Vector<'a, PVTValue>>,
        strings_lookup: &'a StringsLookup,
    ) -> Self {
        let values = match fb_values {
            Some(values) => values.iter().collect(),
            None => Vec::new(),
        };
        Self {
            values,
            strings_lookup,
        }
    }

    pub fn get(&self, i: u32) -> Yaml {
        let v = self.values[i as usize];
        match v.t() {
            PVTValueType::String => self.strings_lookup.get(v.v() as u32),
            PVTValueType::Number => Yaml::Real(v.v().to_string()),
            PVTValueType::Boolean => Yaml::Boolean(if v.v() == 0.0 { false } else { true }),
            _ => Yaml::Real(v.v().to_string()),
        }
    }
}
