use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::actor::Actor;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

#[derive(Clone, Copy)]
pub struct StartPositionsHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for StartPositionsHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("StartPositionsHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> StartPositionsHeader<'a> {
    pub fn new(data: &'a [u8]) -> StartPositionsHeader<'a> {
        StartPositionsHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn actor_list(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, Actor<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}
