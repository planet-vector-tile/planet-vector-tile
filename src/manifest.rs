use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::{collections::BTreeMap, path::PathBuf};

type Layers = BTreeMap<String, Vec<String>>;
type Rules = BTreeMap<String, Rule>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Manifest {
    pub data: Data,
    pub render: Render,
    pub layers: Layers,
    pub rules: Rules,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Data {
    pub source: PathBuf,
    pub planet: PathBuf,
    pub archive: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Render {
    pub leaf_zoom: u8,
    pub layer_order: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Rule {
    pub minzoom: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
}

pub fn parse(path_str: &str) -> Result<Manifest> {
    let path = PathBuf::from(path_str);

    let manifest_str = match std::fs::read_to_string(&path) {
        Ok(manifest) => manifest,
        Err(_) => {
            let msg = format!(
                "No manifest file found at: {}. pwd: {}",
                path.display(),
                std::env::current_dir().unwrap().display()
            );
            return Err(Error::new(ErrorKind::NotFound, msg));
        }
    };

    let mut manifest: Manifest = match serde_yaml::from_str(&manifest_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            let msg = format!("Failed to parse manifest file: {}", e);
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
    };

    let leaf_zoom = manifest.render.leaf_zoom;

    // Leaf zoom must be even
    if leaf_zoom & 1 != 0 {
        let msg = format!("The leaf zoom must be even. leaf_zoom: {}", leaf_zoom);
        return Err(Error::new(ErrorKind::InvalidData, msg));
    }

    // Maximum supported zoom is 14.
    if leaf_zoom > 14 {
        let msg = format!(
            "The maximum supported leaf zoom is 14. leaf_zoom: {}",
            leaf_zoom
        );
        return Err(Error::new(ErrorKind::InvalidData, msg));
    }

    let mut dir = path.clone();
    dir.pop();

    dir = dir.canonicalize()?;

    let mut source = dir.clone();
    let mut planet = dir.clone();
    let mut archive = dir.clone();

    source.push(manifest.data.source);
    planet.push(manifest.data.planet);
    archive.push(manifest.data.archive);

    if !planet.exists() {
        match fs::create_dir_all(&planet) {
            Ok(_) => (),
            Err(e) => {
                let msg = format!(
                    "Unable to create planet directory: {} Err: {:?}",
                    planet.display(),
                    e
                );
                return Err(Error::new(e.kind(), msg));
            }
        }
    }

    // Make the paths in the manifest be relative to the directory of the manifest file.
    // Canonicalize to absolute paths to reduce ambiguity.
    manifest.data.source = source.canonicalize().unwrap_or(source);
    manifest.data.planet = planet.canonicalize().unwrap_or(planet);
    manifest.data.archive = archive.canonicalize().unwrap_or(archive);

    println!("planet dir: {:?}", manifest.data.planet);

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_serde() {
        let mut layers = Layers::new();
        layers.insert(
            "layer0".to_string(),
            vec!["rule0".to_string(), "rule1".to_string()],
        );

        let mut rules = Rules::new();
        let mut tags = BTreeMap::new();
        tags.insert("key0".to_string(), "value0".to_string());
        tags.insert("key1".to_string(), "value1".to_string());
        tags.insert("key2".to_string(), "value2".to_string());
        rules.insert(
            "rule0".to_string(),
            Rule {
                minzoom: 0,
                maxzoom: None,
                keys: vec!["key0".to_string(), "key1".to_string()],
                values: vec!["value0".to_string(), "value1".to_string()],
                tags,
            },
        );

        let m = Manifest {
            data: Data {
                source: PathBuf::from("source"),
                planet: PathBuf::from("planet"),
                archive: PathBuf::from("archive"),
            },
            render: Render {
                leaf_zoom: 12,
                layer_order: vec!["layer0".to_string()],
            },
            layers,
            rules,
        };

        let s = serde_yaml::to_string(&m).unwrap();
        println!("{}", s);
        let m2: Manifest = serde_yaml::from_str(&s).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_reading_manifest() {
        let s = std::fs::read_to_string("manifests/basic.yaml").unwrap();
        let m: Manifest = serde_yaml::from_str(&s).unwrap();
        let s2 = serde_yaml::to_string(&m).unwrap();

        println!("{}", s2);

        assert!(s2.len() > 300);
    }
}
