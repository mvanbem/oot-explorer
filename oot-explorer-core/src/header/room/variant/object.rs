use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::object::ObjectId;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

#[derive(Clone, Copy)]
pub struct ObjectListHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for ObjectListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ObjectListHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> ObjectListHeader<'a> {
    pub fn new(data: &'a [u8]) -> ObjectListHeader<'a> {
        ObjectListHeader { data }
    }

    pub fn count(self) -> u8 {
        self.data[1]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn objects(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, ObjectId> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}
