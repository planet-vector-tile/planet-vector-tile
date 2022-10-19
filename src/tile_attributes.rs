// use std::collections::HashMap;
use indexmap::IndexMap;
use std::hash::{Hash, Hasher};
use std::cell::{Cell, RefCell};

use crate::tile::planet_vector_tile_generated::*;

impl Hash for PVTValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // It is an array of bytes, and we are just using that to hash with.
        self.0.hash(state);
    }
}
impl Eq for PVTValue {}

pub struct TileAttributes {
    // Refactor, as we don't need idx anymore...
    str_idx: Cell<u32>,
    val_idx: Cell<u32>,
    strings: RefCell<IndexMap<String, u32>>,
    values: RefCell<IndexMap<PVTValue, u32>>,
}

impl TileAttributes {
    pub fn new() -> Self {
        TileAttributes {
            str_idx: Cell::new(0),
            val_idx: Cell::new(0),
            strings: RefCell::new(IndexMap::new()),
            values: RefCell::new(IndexMap::new()),
        }
    }

    pub fn upsert_string(&self, str: &str) -> u32 {
        let mut strings = self.strings.borrow_mut();
        match strings.get(str) {
            Some(str_idx) => *str_idx,
            None => {
                let idx = self.str_idx.get();
                strings.insert(String::from(str), idx);
                self.str_idx.set(idx + 1);
                idx
            }
        }
    }

    pub fn upsert_value(&self, value: PVTValue) -> u32 {
        let mut values = self.values.borrow_mut();
        match values.get(&value) {
            Some(val_idx) => *val_idx,
            None => {
                let idx = self.val_idx.get();
                values.insert(value, idx);
                self.val_idx.set(idx + 1);
                idx
            }
        }
    }

    pub fn upsert_string_value(&self, str_val: &str) -> u32 {
        let mut strings = self.strings.borrow_mut();
        let mut values = self.values.borrow_mut();
        match strings.get(str_val) {
            Some(str_idx) => {
                let value = PVTValue::new(PVTValueType::String, *str_idx as f64);
                match values.get(&value) {
                    Some(val_idx) => *val_idx,
                    None => {
                        let idx = self.val_idx.get();
                        values.insert(value, idx);
                        self.val_idx.set(idx + 1);
                        idx
                    }
                }

            },
            None => {
                let str_idx = self.str_idx.get();
                strings.insert(String::from(str_val), str_idx);
                self.str_idx.set(str_idx + 1);
                
                let value = PVTValue::new(PVTValueType::String, str_idx as f64);
                let val_idx = self.val_idx.get();
                values.insert(value, val_idx);
                self.val_idx.set(val_idx + 1);
                val_idx
            }
        }
    }

    // Is there a way we can have a Vec<&str> ?
    pub fn strings(&self) -> Vec<String> {
        let strings = self.strings.borrow();
        let s: Vec<String> = strings.keys().map(|s| String::from(s)).collect();
        s
    }

    // Is there a way we can have a Vec<&PVTValue> ?
    pub fn values(&self) -> Vec<PVTValue> {
        let values = self.values.borrow();
        // let mut value_vec = Vec::<PVTValue>::with_capacity(values.len());
        // for (k, v) in values.iter() {
        //     value_vec[*v as usize] = k.clone();
        // }
        // value_vec
        let v: Vec<PVTValue> = values.keys().map(|v| v.clone()).collect();
        v
    }

}