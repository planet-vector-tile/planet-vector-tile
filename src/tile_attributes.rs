use flatbuffers::{FlatBufferBuilder, WIPOffset, Vector, ForwardsUOffset};
use indexmap::IndexMap;
use std::{hash::{Hash, Hasher}, marker::PhantomData};

use crate::tile::planet_vector_tile_generated::*;

impl Hash for PVTValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // It is an array of bytes, and we are just using that to hash with.
        self.0.hash(state);
    }
}
impl Eq for PVTValue {}

pub struct TileAttributes {
    strings: IndexMap<String, u32>,
    values: IndexMap<PVTValue, u32>,
}

impl TileAttributes {
    pub fn new() -> Self {
        TileAttributes {
            strings: IndexMap::new(),
            values: IndexMap::new(),
        }
    }

    pub fn upsert_string(&mut self, str: &str) -> u32 {
        match self.strings.get(str) {
            Some(str_idx) => *str_idx,
            None => {
                let idx = self.strings.len() as u32;
                self.strings.insert(String::from(str), idx);
                idx
            }
        }
    }

    pub fn upsert_value(&mut self, value: PVTValue) -> u32 {
        match self.values.get(&value) {
            Some(val_idx) => *val_idx,
            None => {
                let idx = self.values.len() as u32;
                self.values.insert(value, idx);
                idx
            }
        }
    }

    pub fn upsert_number_value(&mut self, value: f64) -> u32 {
        self.upsert_value(PVTValue::new(PVTValueType::Number, value))
    }

    pub fn upsert_bool_value(&mut self, value: bool) -> u32 {
        let v = if value { 1_f64 } else { 0_f64 };
        self.upsert_value(PVTValue::new(PVTValueType::Boolean, v))
    }

    pub fn upsert_string_value(&mut self, str_val: &str) -> u32 {
        match self.strings.get(str_val) {
            Some(str_idx) => {
                let value = PVTValue::new(PVTValueType::String, *str_idx as f64);
                match self.values.get(&value) {
                    Some(val_idx) => *val_idx,
                    None => {
                        let idx = self.values.len() as u32;
                        self.values.insert(value, idx);
                        idx
                    }
                }
            }
            None => {
                let str_idx = self.strings.len() as u32;
                self.strings.insert(String::from(str_val), str_idx);
                let value = PVTValue::new(PVTValueType::String, str_idx as f64);
                let val_idx = self.values.len() as u32;
                self.values.insert(value, val_idx);
                val_idx
            }
        }
    }

    pub fn build_strings<'a>(&self, builder: &mut FlatBufferBuilder<'a>) -> WIPOffset<Vector<'a, ForwardsUOffset<&str>>> {
        let len = self.strings.len();
        builder.start_vector::<ForwardsUOffset<&str>>(len);
        for (key, _) in &self.strings {
            let s = builder.create_string(&key);
            builder.push(s);
        }
        builder.end_vector::<ForwardsUOffset<&str>>(len)
    }

    pub fn build_values<'a>(&self, builder: &mut FlatBufferBuilder<'a>) -> WIPOffset<Vector<'a, PVTValue>> {
        let len = self.values.len();
        builder.start_vector::<PVTValue>(len);
        for (value, _) in self.values.iter() {
            builder.push(value);
        }
        builder.end_vector::<PVTValue>(len)
    }

}
