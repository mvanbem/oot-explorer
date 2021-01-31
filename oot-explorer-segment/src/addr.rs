use std::fmt::{self, Debug, Formatter};
// use std::ops::{Add, Sub};

use crate::Segment;

/// A segmented address.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentAddr(pub u32);

impl Debug for SegmentAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "SegmentAddr({:?}, 0x{:06x})",
            self.segment(),
            self.offset(),
        )
    }
}

impl SegmentAddr {
    pub fn segment(self) -> Segment {
        Segment((self.0 >> 24) as u8)
    }

    pub fn offset(self) -> u32 {
        self.0 & 0x00ff_ffff
    }

    pub fn non_null(self) -> Option<SegmentAddr> {
        if self.0 == 0 {
            None
        } else {
            Some(self)
        }
    }
}

// impl Add<u32> for SegmentAddr {
//     type Output = SegmentAddr;

//     fn add(self, rhs: u32) -> SegmentAddr {
//         let result = SegmentAddr(self.0 + rhs);
//         assert_eq!(result.segment(), self.segment());
//         result
//     }
// }

// impl Sub for SegmentAddr {
//     type Output = u32;

//     fn sub(self, rhs: SegmentAddr) -> u32 {
//         assert_eq!(self.segment(), rhs.segment());
//         self.0 - rhs.0
//     }
// }
