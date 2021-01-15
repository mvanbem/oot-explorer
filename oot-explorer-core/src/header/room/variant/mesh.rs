use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::mesh::Mesh;
use crate::segment::{SegmentAddr, SegmentCtx};

#[derive(Clone, Copy)]
pub struct MeshHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for MeshHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("MeshHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}

impl<'a> MeshHeader<'a> {
    pub fn new(data: &'a [u8]) -> MeshHeader<'a> {
        MeshHeader { data }
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn mesh(self, segment_ctx: &SegmentCtx<'a>) -> Mesh<'a> {
        Mesh::new(segment_ctx.resolve(self.ptr()).unwrap())
    }
}
