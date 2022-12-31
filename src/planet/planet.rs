use super::hilbert_pair::{HilbertPair32, HilbertPair40};
use super::metadata::Metadata;
use super::node::Node;
use super::relation::{Member, Relation};
use super::tag::Tag;
use super::way::Way;
use crate::manifest::Manifest;
use crate::mutant::Mutant;
use crate::u40::U40;
use std::fs;
use std::io::Result;
use std::path::PathBuf;
use std::cell::RefCell;

pub struct Planet {
    pub manifest: Manifest,
    pub string_i: RefCell<Mutant<u32>>,
    pub strings: RefCell<Mutant<u8>>,
    pub metadata: RefCell<Mutant<Metadata>>,
    pub tags: RefCell<Mutant<Tag>>,
    pub nodes: RefCell<Mutant<Node>>,
    pub node_tag_i: RefCell<Mutant<u32>>,
    pub ways: RefCell<Mutant<Way>>,
    pub way_tag_i: RefCell<Mutant<u32>>,
    pub refs: RefCell<Mutant<U40>>,
    pub relations: RefCell<Mutant<Relation>>,
    pub relation_tag_i: RefCell<Mutant<u32>>,
    pub members: RefCell<Mutant<Member>>,
    pub roles: RefCell<Mutant<u32>>,
    pub simp_ways: RefCell<Mutant<Way>>,
    pub simp_way_tag_i: RefCell<Mutant<u32>>,
    pub simp_refs: RefCell<Mutant<U40>>,
    pub simp_relations: RefCell<Mutant<Relation>>,
    pub simp_relation_tag_i: RefCell<Mutant<u32>>,
    pub simp_members: RefCell<Mutant<Member>>,

    // These get deleted after the sorting stage is completed during ingest.
    pub node_pairs: Option<RefCell<Mutant<HilbertPair40>>>,
    pub way_pairs: Option<RefCell<Mutant<HilbertPair32>>>,
    pub relation_pairs: Option<RefCell<Mutant<HilbertPair32>>>,
}

impl Planet {
    pub fn new(manifest: &Manifest) -> Result<Planet> {
        let dir = &manifest.data.planet;

        // Copy the manifest to the build directory so we know exactly what it was at the time of build.
        let mut planet_manifest = manifest.clone();
        planet_manifest.data.planet = PathBuf::from("./");
        let manifest_str = serde_yaml::to_string(&planet_manifest)
            .expect("Cound not re-serialize manifest to planet dir.");
        fs::write(dir.join("manifest.yaml"), manifest_str)?;

        let string_i = RefCell::new(Mutant::<u32>::new_empty(dir, "string_i")?);
        let strings = RefCell::new(Mutant::<u8>::new_empty(dir, "strings")?);
        let metadata = RefCell::new(Mutant::<Metadata>::new_empty(dir, "metadata")?);
        let tags = RefCell::new(Mutant::<Tag>::new_empty(dir, "tags")?);
        let nodes = RefCell::new(Mutant::<Node>::new_empty(dir, "nodes")?);
        let node_tag_i = RefCell::new(Mutant::<u32>::new_empty(dir, "node_tags")?);
        let ways = RefCell::new(Mutant::<Way>::new_empty(dir, "ways")?);
        let way_tag_i = RefCell::new(Mutant::<u32>::new_empty(dir, "way_tags")?);
        let refs = RefCell::new(Mutant::<U40>::new_empty(dir, "refs")?);
        let relations = RefCell::new(Mutant::<Relation>::new_empty(dir, "relations")?);
        let relation_tag_i = RefCell::new(Mutant::<u32>::new_empty(dir, "relation_tags")?);
        let members = RefCell::new(Mutant::<Member>::new_empty(dir, "members")?);
        let roles = RefCell::new(Mutant::<u32>::new_empty(dir, "roles")?);
        let simp_ways = RefCell::new(Mutant::<Way>::new_empty(dir, "simp_ways")?);
        let simp_way_tag_i = RefCell::new(Mutant::<u32>::new_empty(dir, "simp_way_tag_i")?);
        let simp_refs = RefCell::new(Mutant::<U40>::new_empty(dir, "simp_refs")?);
        let simp_relations = RefCell::new(Mutant::<Relation>::new_empty(dir, "simp_relations")?);
        let simp_relation_tag_i = RefCell::new(Mutant::<u32>::new_empty(dir, "simp_relation_tag_i")?);
        let simp_members = RefCell::new(Mutant::<Member>::new_empty(dir, "simp_members")?);

        let node_pairs = Some(RefCell::new(Mutant::<HilbertPair40>::new_empty(dir, "node_pairs")?));
        let way_pairs = Some(RefCell::new(Mutant::<HilbertPair32>::new_empty(dir, "way_pairs")?));
        let relation_pairs = Some(RefCell::new(Mutant::<HilbertPair32>::new_empty(dir, "relation_pairs")?));

        Ok(Self {
            manifest: planet_manifest,
            string_i,
            strings,
            metadata,
            tags,
            nodes,
            node_tag_i,
            ways,
            way_tag_i,
            refs,
            relations,
            relation_tag_i,
            members,
            roles,
            simp_ways,
            simp_way_tag_i,
            simp_refs,
            simp_relations,
            simp_relation_tag_i,
            simp_members,
            node_pairs,
            way_pairs,
            relation_pairs,
        })
    }

    pub fn open(manifest: &Manifest) -> Result<Planet> {
        let dir = &manifest.data.planet;

        let string_i = RefCell::new(Mutant::<u32>::open(dir, "string_i", false)?);
        let strings = RefCell::new(Mutant::<u8>::open(dir, "strings", false)?);
        let metadata = RefCell::new(Mutant::<Metadata>::open(dir, "metadata", false)?);
        let tags = RefCell::new(Mutant::<Tag>::open(dir, "tags", false)?);
        let nodes = RefCell::new(Mutant::<Node>::open(dir, "nodes", false)?);
        let node_tag_i = RefCell::new(Mutant::<u32>::open(dir, "node_tags", false)?);
        let ways = RefCell::new(Mutant::<Way>::open(dir, "ways", false)?);
        let way_tag_i = RefCell::new(Mutant::<u32>::open(dir, "way_tags", false)?);
        let refs = RefCell::new(Mutant::<U40>::open(dir, "refs", false)?);
        let relations = RefCell::new(Mutant::<Relation>::open(dir, "relations", false)?);
        let relation_tag_i = RefCell::new(Mutant::<u32>::open(dir, "relation_tags", false)?);
        let members = RefCell::new(Mutant::<Member>::open(dir, "members", false)?);
        let roles = RefCell::new(Mutant::<u32>::open(dir, "roles", false)?);
        let simp_ways = RefCell::new(Mutant::<Way>::open(dir, "simp_ways", false)?);
        let simp_way_tag_i = RefCell::new(Mutant::<u32>::open(dir, "simp_way_tag_i", false)?);
        let simp_refs = RefCell::new(Mutant::<U40>::open(dir, "simp_refs", false)?);
        let simp_relations = RefCell::new(Mutant::<Relation>::open(dir, "simp_relations", false)?);
        let simp_relation_tag_i = RefCell::new(Mutant::<u32>::open(dir, "simp_relation_tag_i", false)?);
        let simp_members = RefCell::new(Mutant::<Member>::open(dir, "simp_members", false)?);

        let node_pairs = match Mutant::<HilbertPair40>::open(dir, "node_pairs", false) {
            Ok(mutant) => Some(RefCell::new(mutant)),
            Err(_) => None,
        };
        let way_pairs = match Mutant::<HilbertPair32>::open(dir, "way_pairs", false) {
            Ok(mutant) => Some(RefCell::new(mutant)),
            Err(_) => None,
        };
        let relation_pairs = match Mutant::<HilbertPair32>::open(dir, "relation_pairs", false) {
            Ok(mutant) => Some(RefCell::new(mutant)),
            Err(_) => None,
        };

        Ok(Self {
            manifest: manifest.clone(),
            string_i,
            strings,
            metadata,
            tags,
            nodes,
            node_tag_i,
            ways,
            way_tag_i,
            refs,
            relations,
            relation_tag_i,
            members,
            roles,
            simp_ways,
            simp_way_tag_i,
            simp_refs,
            simp_relations,
            simp_relation_tag_i,
            simp_members,
            node_pairs,
            way_pairs,
            relation_pairs,
        })
    }

    pub fn slice(&self) -> PlanetSlice {
        let string_i = self.string_i.borrow().slice();
        let strings = self.strings.borrow().slice();
        let metadata = self.metadata.borrow().slice();
        let tags = self.tags.borrow().slice();
        let nodes = self.nodes.borrow().slice();
        let node_tag_i = self.node_tag_i.borrow().slice();
        let ways = self.ways.borrow().slice();
        let way_tag_i = self.way_tag_i.borrow().slice();
        let refs = self.refs.borrow().slice();
        let relations = self.relations.borrow().slice();
        let relation_tag_i = self.relation_tag_i.borrow().slice();
        let members = self.members.borrow().slice();
        let roles = self.roles.borrow().slice();

        let simp_ways = self.simp_ways.borrow().slice();
        let simp_way_tag_i = self.simp_way_tag_i.borrow().slice();
        let simp_refs = self.simp_refs.borrow().slice();
        let simp_relations = self.simp_relations.borrow().slice();
        let simp_relation_tag_i = self.simp_relation_tag_i.borrow().slice();
        let simp_members = self.simp_members.borrow().slice();

        let node_pairs = match &self.node_pairs {
            Some(m) => Some(m.borrow().slice()),
            None => None,
        };
        let way_pairs = match &self.way_pairs {
            Some(m) => Some(m.borrow().slice()),
            None => None,
        };
        let relation_pairs = match &self.relation_pairs {
            Some(m) => Some(m.borrow().slice()),
            None => None,
        };

        PlanetSlice {
            string_i,
            strings,
            metadata,
            tags,
            nodes,
            node_tag_i,
            ways,
            way_tag_i,
            refs,
            relations,
            relation_tag_i,
            members,
            roles,
            simp_ways,
            simp_way_tag_i,
            simp_refs,
            simp_relations,
            simp_relation_tag_i,
            simp_members,
            node_pairs,
            way_pairs,
            relation_pairs,
        }
    }

    pub fn mutable_slice(&self) -> PlanetMutableSlice {
        let string_i = self.string_i.borrow().mutable_slice();
        let strings = self.strings.borrow().mutable_slice();
        let metadata = self.metadata.borrow().mutable_slice();
        let tags = self.tags.borrow().mutable_slice();
        let nodes = self.nodes.borrow().mutable_slice();
        let node_tag_i = self.node_tag_i.borrow().mutable_slice();
        let ways = self.ways.borrow().mutable_slice();
        let way_tag_i = self.way_tag_i.borrow().mutable_slice();
        let refs = self.refs.borrow().mutable_slice();
        let relations = self.relations.borrow().mutable_slice();
        let relation_tag_i = self.relation_tag_i.borrow().mutable_slice();
        let members = self.members.borrow().mutable_slice();
        let roles = self.roles.borrow().mutable_slice();
        let simp_ways = self.simp_ways.borrow().mutable_slice();
        let simp_way_tag_i = self.simp_way_tag_i.borrow().mutable_slice();
        let simp_refs = self.simp_refs.borrow().mutable_slice();
        let simp_relations = self.simp_relations.borrow().mutable_slice();
        let simp_relation_tag_i = self.simp_relation_tag_i.borrow().mutable_slice();
        let simp_members = self.simp_members.borrow().mutable_slice();
        let node_pairs = match &self.node_pairs {
            Some(m) => Some(m.borrow().mutable_slice()),
            None => None,
        };
        let way_pairs = match &self.way_pairs {
            Some(m) => Some(m.borrow().mutable_slice()),
            None => None,
        };
        let relation_pairs = match &self.relation_pairs {
            Some(m) => Some(m.borrow().mutable_slice()),
            None => None,
        };

        PlanetMutableSlice {
            string_i,
            strings,
            metadata,
            tags,
            nodes,
            node_tag_i,
            ways,
            way_tag_i,
            refs,
            relations,
            relation_tag_i,
            members,
            roles,
            simp_ways,
            simp_way_tag_i,
            simp_refs,
            simp_relations,
            simp_relation_tag_i,
            simp_members,
            node_pairs,
            way_pairs,
            relation_pairs,
        }
    }
}

pub struct PlanetSlice<'a> {
    pub string_i: &'a [u32],
    pub strings: &'a [u8],
    pub metadata: &'a [Metadata],
    pub tags: &'a [Tag],
    pub nodes: &'a [Node],
    pub node_tag_i: &'a [u32],
    pub ways: &'a [Way],
    pub way_tag_i: &'a [u32],
    pub refs: &'a [U40],
    pub relations: &'a [Relation],
    pub relation_tag_i: &'a [u32],
    pub members: &'a [Member],
    pub roles: &'a [u32],
    pub simp_ways: &'a [Way],
    pub simp_way_tag_i: &'a [u32],
    pub simp_refs: &'a [U40],
    pub simp_relations: &'a [Relation],
    pub simp_relation_tag_i: &'a [u32],
    pub simp_members: &'a [Member],
    pub node_pairs: Option<&'a [HilbertPair40]>,
    pub way_pairs: Option<&'a [HilbertPair32]>,
    pub relation_pairs: Option<&'a [HilbertPair32]>,
}

pub struct PlanetMutableSlice<'a> {
    pub string_i: &'a mut [u32],
    pub strings: &'a mut [u8],
    pub metadata: &'a mut [Metadata],
    pub tags: &'a mut [Tag],
    pub nodes: &'a mut [Node],
    pub node_tag_i: &'a mut [u32],
    pub ways: &'a mut [Way],
    pub way_tag_i: &'a mut [u32],
    pub refs: &'a mut [U40],
    pub relations: &'a mut [Relation],
    pub relation_tag_i: &'a mut [u32],
    pub members: &'a mut [Member],
    pub roles: &'a mut [u32],
    pub simp_ways: &'a mut [Way],
    pub simp_way_tag_i: &'a mut [u32],
    pub simp_refs: &'a mut [U40],
    pub simp_relations: &'a mut [Relation],
    pub simp_relation_tag_i: &'a mut [u32],
    pub simp_members: &'a mut [Member],
    pub node_pairs: Option<&'a mut [HilbertPair40]>,
    pub way_pairs: Option<&'a mut [HilbertPair32]>,
    pub relation_pairs: Option<&'a mut [HilbertPair32]>,
}
