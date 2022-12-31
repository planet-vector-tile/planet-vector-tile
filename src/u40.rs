#[repr(packed)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct U40 {
    x: [u8; 5],
}

impl U40 {
    pub fn from_u64(x: u64) -> Self {
        let x = x.to_le_bytes();
        debug_assert_eq!((x[5], x[6], x[7]), (0, 0, 0));
        Self {
            x: [x[0], x[1], x[2], x[3], x[4]],
        }
    }

    pub fn to_u64(self) -> u64 {
        let extented = [
            self.x[0], self.x[1], self.x[2], self.x[3], self.x[4], 0, 0, 0,
        ];
        u64::from_le_bytes(extented)
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for U40 {
    fn hash<H>(&self, h: &mut H)
    where
        H: std::hash::Hasher,
    {
        // We manually implement Hash like this, since [u8; 5] is slower to hash
        // than u64 for some/many hash functions
        self.to_u64().hash(h)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_basic_u40() {
//         let a: u64 = 0x1FFFFFFFF;
//         // println!("{:x}", a);
//         let u40 = U40::new(a);
//         // println!("{:?}", u40);
//         let h = u40.high;
//         let l = u40.low;
//         assert_eq!(h, 0x1);
//         assert_eq!(l, 0xFFFFFFFF);
//         // println!("high {:x} low {:x}", h, l);
//         let b = u40.v();
//         // println!("{:x}", b);
//         assert_eq!(a, b);
//     }
// }
