use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::segment::SegmentAddr;

#[derive(Clone, Copy)]
pub struct AlternateHeadersHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for AlternateHeadersHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("AlternateHeadersHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> AlternateHeadersHeader<'a> {
    pub fn new(data: &'a [u8]) -> AlternateHeadersHeader<'a> {
        AlternateHeadersHeader { data }
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    // TODO: Expose and parse these! Count is not explicitly stored.
}
