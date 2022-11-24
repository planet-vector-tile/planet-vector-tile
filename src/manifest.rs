use std::{collections::BTreeMap, path::{Path, PathBuf}};

use serde_derive::{Deserialize, Serialize};

pub type Layers = BTreeMap<String, Vec<String>>;
pub type Rules = BTreeMap<String, Rule>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Manifest {
    pub render: Render,
    pub layers: Layers,
    pub rules: Rules,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Render {
    pub leaf_zoom: u8,
    pub layer_order: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Rule {
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<(String, String)>,
}

pub fn parse(path: &Option<PathBuf>) -> Manifest {
    let default = PathBuf::from("./manifest.json");

    let manifest_path = match path {
        Some(manifest) => manifest,
        None => &default,
    };

    let manifest_str = match std::fs::read_to_string(manifest_path) {
        Ok(manifest) => manifest,
        Err(_) => {
            log::warn!("No manifest file found at {}", manifest_path.display());
            std::process::exit(1);
        }
    };

    let manifest: Manifest = match toml::from_str(&manifest_str) {
        Ok(manifest) => manifest,
        Err(e) => {
            log::warn!("Failed to parse manifest file: {}", e);
            std::process::exit(1);
        }
    };

    manifest
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
        rules.insert(
            "rule0".to_string(),
            Rule {
                minzoom: 0,
                maxzoom: None,
                keys: vec!["key0".to_string(), "key1".to_string()],
                values: vec!["value0".to_string(), "value1".to_string()],
                tags: vec![
                    ("key0".to_string(), "value0".to_string()).into(),
                    ("key1".to_string(), "value1".to_string()).into(),
                ],
            },
        );

        let m = Manifest {
            render: Render {
                leaf_zoom: 12,
                layer_order: vec!["layer0".to_string()],
            },
            layers,
            rules,
        };

        let s = toml::to_string(&m).unwrap();
        println!("{}", s);
        let m2: Manifest = toml::from_str(&s).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_reading_manifest() {
        let s = std::fs::read_to_string("manifest.toml").unwrap();
        let m: Manifest = toml::from_str(&s).unwrap();
        let s2 = toml::to_string(&m).unwrap();

        assert!(s2.len() > 500);
    }
}
