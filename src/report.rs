
pub struct ReportOptions {
    pub entities_by_rule: BTreeSet<String>,
    pub view_tags: BTreeSet<String>,

}

pub struct Report {
    pub rule_stats: BTreeMap<String, RuleStat>,
    pub entities_by_rule: BTreeMap<String, Vec<Entity>>,
}

pub struct Entity {
    pub nwr: EntityType,
    pub osm_id: u64,
    pub tags: BTreeMap<String, String>,
    pub rules: Vec<String>,
}

pub enum EntityType {
    Node,
    Way,
    Relation,
}

pub struct RuleStat {
    pub name: String,
    pub count: u64,
    pub n: u64,
    pub w: u64,
    pub r: u64,
}
