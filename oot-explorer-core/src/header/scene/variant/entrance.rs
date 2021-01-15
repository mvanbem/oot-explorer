use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::entrance::entry::Entrance;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

pub mod entry;

#[derive(Clone, Copy)]
pub struct EntranceListHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for EntranceListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("EntranceListHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> EntranceListHeader<'a> {
    pub fn new(data: &'a [u8]) -> EntranceListHeader<'a> {
        EntranceListHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn entrance_list_with_len(
        self,
        segment_ctx: &'a SegmentCtx,
        len: usize,
    ) -> Slice<'a, Entrance<'a>> {
        Slice::new(segment_ctx.resolve(self.ptr()).unwrap(), len)
    }
}
