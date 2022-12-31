use std::path::Path;

use crate::mutant::Mutant;
use ahash::AHashMap;
use std::io::Result;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Relation {
    pub id: u32,
    pub tag_i: u32,
    pub member_i: u32,
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct RelationMember {
    pub ref_i: u32,
    pub pack: u8,
    pub role_i: u8,
}

impl RelationMember {
    pub fn new(ref_entity: RelationMemberEntity, role_i: u8) -> Self {
        match ref_entity {
            RelationMemberEntity::Unresolved => Self {
                ref_i: 0,
                pack: 0,
                role_i,
            },
            RelationMemberEntity::Node(ref_i) => {
                let upper = (ref_i >> 32) as u8;
                Self {
                    ref_i: ref_i as u32,
                    pack: upper << 4 | 1,
                    role_i,
                }
            }
            RelationMemberEntity::Way(ref_i) => Self {
                ref_i,
                pack: 2,
                role_i,
            },
            RelationMemberEntity::Relation(ref_i) => Self {
                ref_i,
                pack: 3,
                role_i,
            },
        }
    }
    pub fn entity(&self) -> RelationMemberEntity {
        match self.pack & 0b1111 {
            0 => RelationMemberEntity::Unresolved,
            1 => {
                let upper = self.pack as u64 >> 4;
                let ref_i = upper << 32 & self.ref_i as u64;
                RelationMemberEntity::Node(ref_i)
            }
            2 => RelationMemberEntity::Way(self.ref_i),
            3 => RelationMemberEntity::Relation(self.ref_i),
            _ => panic!("Invalid RelationMember pack"),
        }
    }
}

pub enum RelationMemberEntity {
    Unresolved,
    Node(u64),
    Way(u32),
    Relation(u32),
}

pub struct Roles {
    pub string_ref_to_role_idx: AHashMap<u32, u8>,
}

impl Roles {
    pub fn new() -> Self {
        Self {
            string_ref_to_role_idx: AHashMap::new(),
        }
    }
    pub fn upsert(&mut self, string_ref: u32) -> u8 {
        if let Some(role_idx) = self.string_ref_to_role_idx.get(&string_ref) {
            *role_idx
        } else {
            let role_idx = self.string_ref_to_role_idx.len() as u8;
            self.string_ref_to_role_idx.insert(string_ref, role_idx);
            role_idx
        }
    }
    pub fn write(&self, dir: &Path, file_name: &str) -> Result<()> {
        let mutant = Mutant::<u32>::new(dir, file_name, self.string_ref_to_role_idx.len())?;
        let vec = mutant.mutable_slice();
        for (string_ref, role_i) in self.string_ref_to_role_idx.iter() {
            vec[*role_i as usize] = *string_ref;
        }
        Ok(())
    }
}
