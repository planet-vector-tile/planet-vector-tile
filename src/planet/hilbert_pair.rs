use super::u40::U40;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct HilbertPair40 {
    pub h: u64,
    pub i: U40,
}

impl HilbertPair40 {
    pub fn new(h: u64, i: u64) -> Self {
        Self {
            h,
            i: U40::from_u64(i),
        }
    }
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct HilbertPair32 {
    pub h: u64,
    pub i: u32,
}

impl HilbertPair32 {
    pub fn new(h: u64, i: u32) -> Self {
        Self { h, i }
    }
}
