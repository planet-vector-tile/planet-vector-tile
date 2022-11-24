use std::collections::BTreeMap;

use serde_derive::{Deserialize, Serialize};

pub type Layers = BTreeMap<String, Vec<String>>;
pub type Rules = BTreeMap<String, Rule>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Manifest {
    render: Render,
    layers: Layers,
    rules: Rules,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Render {
    leaf_zoom: u8,
    layer_order: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Rule {
    minzoom: u8,
    maxzoom: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<(String, String)>,
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
