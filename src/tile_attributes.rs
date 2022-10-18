use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[allow(dead_code, unused_imports)]
#[path = "./fbs/planet_vector_tile_generated.rs"]
mod planet_vector_tile_generated;
use planet_vector_tile_generated::*;

impl Hash for PVTValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // It is an array of bytes, and we are just using that to hash with.
        self.0.hash(state);
    }
}
impl Eq for PVTValue {}

pub struct TileAttributes {
    str_idx: u32,
    val_idx: u32,
    strings: HashMap<String, u32>,
    values: HashMap<PVTValue, u32>,
}

impl<'a> TileAttributes {
    pub fn new() -> Self {
        TileAttributes {
            str_idx: 0,
            val_idx: 0,
            strings: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn upsert_string(&mut self, str: &str) -> u32 {
        match self.strings.get(str) {
            Some(str_idx) => *str_idx,
            None => {
                let idx = self.str_idx;
                self.strings.insert(String::from(str), idx);
                self.str_idx += 1;
                idx
            }
        }
    }

    pub fn upsert_value(&mut self, value: PVTValue) -> u32 {
        match self.values.get(&value) {
            Some(val_idx) => *val_idx,
            None => {
                let idx = self.val_idx;
                self.values.insert(value, idx);
                self.val_idx += 1;
                idx
            }
        }
    }

    pub fn upsert_string_value(&mut self, str_val: &str) -> u32 {
        match self.strings.get(str_val) {
            Some(str_idx) => {
                let value = PVTValue::new(PVTValueType::String, *str_idx as f64);
                match self.values.get(&value) {
                    Some(val_idx) => *val_idx,
                    None => {
                        let idx = self.val_idx;
                        self.values.insert(value, idx);
                        self.val_idx += 1;
                        idx
                    }
                }

            },
            None => {
                let str_idx = self.str_idx;
                self.strings.insert(String::from(str_val), str_idx);
                self.str_idx += 1;
                
                let value = PVTValue::new(PVTValueType::String, str_idx as f64);
                let val_idx = self.val_idx;
                self.values.insert(value, val_idx);
                val_idx
            }
        }
    }

    pub fn strings(&'a self) -> Vec<&'a str> {
        let mut strings = Vec::<&'a str>::with_capacity(self.strings.len());
        for (k, v) in self.strings.iter() {
            strings[*v as usize] = k;
        }
        strings
    }

    pub fn values(&'a self) -> Vec<&'a PVTValue> {
        let mut values = Vec::<&'a PVTValue>::with_capacity(self.values.len());
        for (k, v) in self.values.iter() {
            values[*v as usize] = k;
        }
        values
    }

}