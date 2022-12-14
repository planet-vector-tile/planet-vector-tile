
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct U40 {
    pub low: u32,
    pub high: u8
}

impl U40 {
    pub fn new(v: u64) -> Self {
        Self {
            low: v as u32,
            high: (v >> 8) as u8
        }
    }

    pub fn v(&self) -> u64 {
        (self.high as u64) << 8 & self.low as u64
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_u40() {
        let a: u64 = 0x1FFFFFFFF;
        let u40 = U40::new(a);
        println!("{:?}", u40);
        let b = u40.v();
        println!("{}", b);
    }

}