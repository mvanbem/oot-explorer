use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::collision::Collision;
use crate::segment::{SegmentAddr, SegmentCtx};

#[derive(Clone, Copy)]
pub struct CollisionHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for CollisionHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CollisionHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> CollisionHeader<'a> {
    pub fn new(data: &'a [u8]) -> CollisionHeader<'a> {
        CollisionHeader { data }
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn collision(self, segment_ctx: &'a SegmentCtx) -> Collision<'a> {
        Collision::new(segment_ctx.resolve(self.ptr()).unwrap())
    }
}
