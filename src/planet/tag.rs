use ahash::AHashMap;
use std::{io, collections::hash_map};

use super::planet::Planet;
use super::EntityType;

type Error = Box<dyn std::error::Error>;

pub struct Tag {
    pub k: u32,
    pub v: u32,
}

/// Holds tags external vector and deduplicates tags.
pub struct TagSerializer<'a> {
    planet: &'a Planet,
    dedup: AHashMap<(u32, u32), u32>, // deduplication table: (key_idx, val_idx) -> pos
}

impl<'a> TagSerializer<'a> {
    pub fn new(planet: &'a Planet) -> io::Result<Self> {
        Ok(Self {
            planet,
            dedup: AHashMap::new(),
        })
    }

    pub fn serialize(&mut self, entity_type: EntityType, key_idx: u32, val_idx: u32) -> Result<(), Error> {
        let i = match self
            .dedup
            .entry((key_idx, val_idx))
        {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let tag = Tag {
                    k: key_idx,
                    v: val_idx,
                };
                let mut tags = self.planet.tags.borrow_mut();
                tags.push(tag);
                let i = tags.len as u32;
                entry.insert(i);
                i
            }
        };

        match entity_type {
            EntityType::Node => self.planet.node_tag_i.borrow_mut().push(i),
            EntityType::Way => self.planet.way_tag_i.borrow_mut().push(i),
            EntityType::Relation => self.planet.relation_tag_i.borrow_mut().push(i),
        };

        Ok(())
    }
}