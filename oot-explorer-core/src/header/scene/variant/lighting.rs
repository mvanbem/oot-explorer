use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::scene::Lighting;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

#[derive(Clone, Copy)]
pub struct LightingHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for LightingHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("LightingHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> LightingHeader<'a> {
    pub fn new(data: &'a [u8]) -> LightingHeader<'a> {
        LightingHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn lighting(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, Lighting<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}
