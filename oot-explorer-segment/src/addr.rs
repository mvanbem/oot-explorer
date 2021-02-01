use std::fmt::{self, Debug, Formatter};

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

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn non_null(self) -> Option<SegmentAddr> {
        if self.0 == 0 {
            None
        } else {
            Some(self)
        }
    }
}
