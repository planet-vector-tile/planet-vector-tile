
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Manifest {
    layers: Vec<Layer>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EntityType {
    Node,
    Way,
    Relation,
    Any,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Layer {
    name: String,
    minzoom: u8,
    maxzoom: Option<u8>,
    entity_type: EntityType,
    keys: Vec<String>,
    tags: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_serde() {
        let m = Manifest {
            layers: vec![Layer {
                name: "test".to_string(),
                minzoom: 0,
                maxzoom: None,
                entity_type: EntityType::Node,
                keys: vec!["key1".to_string(), "key2".to_string()],
                tags: vec![
                    ("key1".to_string(), "value1".to_string()),
                    ("key2".to_string(), "value2".to_string()),
                ],
            }],
        };

        let s = toml::to_string(&m).unwrap();
        println!("{}", s);
        let m2: Manifest = toml::from_str(&s).unwrap();
        assert_eq!(m, m2);
    }
}
