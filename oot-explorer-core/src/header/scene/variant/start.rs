use byteorder::{BigEndian, ReadBytesExt};

use crate::header::actor::{Actor, ACTOR_DESC};
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::struct_::{FieldDescriptor, StructFieldLocation, VariantDescriptor};
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::Slice;

pub const START_POSITIONS_DESC: &VariantDescriptor = &VariantDescriptor {
    fields: &[FieldDescriptor {
        name: "start_positions",
        location: StructFieldLocation::Slice {
            count_offset: 1,
            count_desc: PrimitiveType::U8,
            ptr_offset: 4,
        },
        desc: ACTOR_DESC,
    }],
};

#[derive(Clone, Copy)]
pub struct StartPositionsHeader<'a> {
    data: &'a [u8],
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
