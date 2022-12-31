use ahash::AHashMap;

use super::planet::Planet;
use std::collections::BTreeMap;
use std::collections::hash_map::Entry;
use std::str;
use std::io::{Error, ErrorKind, Result};

pub struct Strings<'a> {
    pub string_i: &'a [u32],
    pub strings: &'a [u8],
}

impl<'a> Strings<'a> {
    pub fn new(planet: &'a Planet) -> Self {
        Self {
            string_i: planet.string_i.slice(),
            strings: planet.strings.slice(),
        }
    }

    pub fn get(&self, i: usize) -> &str {
        if i == self.string_i.len() - 1 {
            let bytes = &self.strings[i..];
            let str = unsafe { str::from_utf8_unchecked(bytes) };
            return str;
        }
        let start = self.string_i[i] as usize;
        let end = self.string_i[i + 1] as usize;
        let bytes = &self.strings[start..end];
        let str = unsafe { str::from_utf8_unchecked(bytes) };
        str
    }
}

pub struct StringBuilder {
    str_to_count: AHashMap<String, u32>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self {
            str_to_count: AHashMap::new(),
        }
    }

    pub fn new_full_planet() -> Self {
        StringBuilder::with_capacity(114520000) // planet benchmarked to 114_514_897
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            str_to_count: AHashMap::with_capacity(capacity),
        }
    }

    pub fn upsert(&mut self, str: &str) {
        let s = str.to_string();
        match self.str_to_count.entry(s) {
            Entry::Occupied(mut o) => {
                o.insert(o.get() + 1);
            }
            Entry::Vacant(v) => {
                v.insert(1);
            }
        }
    }

    pub fn serialize(&self, planet: &mut Planet) -> Result<()> {
        let mut count_to_str: BTreeMap<u32, Vec<&str>> = BTreeMap::new();

        for (string, count) in &self.str_to_count {
            match count_to_str.entry(*count) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(vec![string]);
                },
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let strings = entry.get_mut();
                    strings.push(string)
                },
            }
        }

        let m_string_i = &mut planet.string_i;
        let m_strings = &mut planet.strings;

        for (count, strings) in count_to_str.iter().rev() {
            println!("count {}", count);
            for s in strings {
                println!("    {}", s);
                let bytes = s.as_bytes();
                m_strings.append(bytes)?;
                m_string_i.push(m_strings.len as u32);
            }
        }

        Ok(())
    }
}
