use super::{node::Node, planet::PlanetSlice, tag::Tag};

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Way {
    id: u32,
}

impl Way {
    pub fn tags<'a>(&self, planet: &'a PlanetSlice) -> &'a [Tag] {
        planet.tags
    }

    pub fn nodes<'a>(&self, planet: &'a PlanetSlice) -> &'a [Node] {
        planet.nodes
    }

    pub fn way_type(&self, planet: &PlanetSlice) -> WayType {
        WayType::Line
    }
}

pub enum WayType {
    Line,
    ClosedLine,
    Polygon,
}
