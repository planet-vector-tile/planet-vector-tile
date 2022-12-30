
pub struct Relation {
    pub h_merc: u64,
    pub osm_id: u32,
    pub tags_i: u32
    pub members_i: u32,
}

pub struct HilberRelationPair {
    pub h_merc: u64,
    pub i: u32,
}

pub struct RelationMember {
    pub ref_i: u32,
    pub pack: u8,
    pub role_i: u8,
}

impl RelationMember {
    pub fn entity(&self) -> RelationMemberEntity {
        let upper = ((self.pack >> 2) & 0b11) << 6;
        let ref_i = upper & self.ref_i as usize;
        match self.pack & 0b11 {
            0 => {
                // pack 2 more bytes for a node ref, as node_i may require up to 34 bits.
                let upper = ((self.pack >> 2) & 0b11) << 6;
                let ref_i = upper & self.ref_i as usize;
                RelationMemberEntity::Node(ref_i)
            },
            1 => RelationMemberEntity::Way(self.ref_i),
            2 => RelationMemberEntity::Relation(self.ref_i),
            _ => panic!("Invalid RelationMember pack"),
        }
    }

}

pub enum RelationMemberEntity {
    Node(usize),
    Way(usize),
    Relation(usize),
}
