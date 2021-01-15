use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::exit::entry::Exit;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

pub mod entry;

#[derive(Clone, Copy)]
pub struct ExitListHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for ExitListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ExitListHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> ExitListHeader<'a> {
    pub fn new(data: &'a [u8]) -> ExitListHeader<'a> {
        ExitListHeader { data }
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn exit_list_with_len(
        self,
        segment_ctx: &'a SegmentCtx,
        len: usize,
    ) -> Slice<'a, Exit<'a>> {
        Slice::new(segment_ctx.resolve(self.ptr()).unwrap(), len)
    }
}
