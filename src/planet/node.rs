use super::u40::U40;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Node {
    id: U40,
    tag_i: u32,
}
