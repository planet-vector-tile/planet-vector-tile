use crate::u40::U40;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Node {
    h: u64,
    id: U40,
    lon: i32,
    lat: i32,
    tag_i: u32,
}
