use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::transition::entry::TransitionActor;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

pub mod entry;

#[derive(Clone, Copy)]
pub struct TransitionActorsHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for TransitionActorsHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TransitionActorsHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> TransitionActorsHeader<'a> {
    pub fn new(data: &'a [u8]) -> TransitionActorsHeader<'a> {
        TransitionActorsHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn transition_actors(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, TransitionActor<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}
