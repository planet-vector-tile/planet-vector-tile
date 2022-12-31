use crate::u40::U40;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Node {
    h: u64,
    lon: i32,
    lat: i32,
    id: U40,
    tag_i: u32,
}
