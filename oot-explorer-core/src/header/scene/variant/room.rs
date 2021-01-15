use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::room::entry::RoomListEntry;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

pub mod entry;

#[derive(Clone, Copy)]
pub struct RoomListHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for RoomListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomListHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> RoomListHeader<'a> {
    pub fn new(data: &'a [u8]) -> RoomListHeader<'a> {
        RoomListHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn room_list(self, segment_ctx: &SegmentCtx<'a>) -> Slice<'a, RoomListEntry<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}
