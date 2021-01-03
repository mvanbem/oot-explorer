use crate::gbi::DisplayList;
use crate::segment::{SegmentAddr, SegmentCtx, SegmentResolveError};
use crate::slice::{Slice, StructReader};
use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};
use std::ops::RangeInclusive;

#[derive(Clone, Copy)]
pub struct Mesh<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Mesh<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Mesh")
            .field("type_", &self.type_())
            .finish()
    }
}
impl<'a> Mesh<'a> {
    pub fn new(data: &'a [u8]) -> Mesh<'a> {
        Mesh { data }
    }
    pub fn type_(self) -> u8 {
        self.data[0]
    }
    pub fn variant(self) -> MeshVariant<'a> {
        let data = self.data;
        match self.type_() {
            0x00 => MeshVariant::Simple(SimpleMesh { data }),
            0x01 => MeshVariant::Jfif(JfifMesh { _data: data }),
            0x02 => MeshVariant::Clipped(ClippedMesh { data }),
            type_ => panic!("unexpected mesh type: 0x{:02x}", type_),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MeshVariant<'a> {
    Simple(SimpleMesh<'a>),
    Jfif(JfifMesh<'a>),
    Clipped(ClippedMesh<'a>),
}

#[derive(Clone, Copy)]
pub struct SimpleMesh<'a> {
    data: &'a [u8],
}
impl<'a> std::fmt::Debug for SimpleMesh<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SimpleMesh").finish()
    }
}
impl<'a> SimpleMesh<'a> {
    pub fn new(data: &'a [u8]) -> SimpleMesh<'a> {
        SimpleMesh { data }
    }
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn start(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn end(self) -> SegmentAddr {
        SegmentAddr((&self.data[8..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn entries(self, segment_ctx: &SegmentCtx<'a>) -> Slice<'a, SimpleMeshEntry<'a>> {
        Slice::new(
            segment_ctx.resolve_range(self.start()..self.end()).unwrap(),
            self.count() as usize,
        )
    }
}

#[derive(Clone, Copy)]
pub struct SimpleMeshEntry<'a> {
    data: &'a [u8],
}
impl<'a> std::fmt::Debug for SimpleMeshEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SimpleMeshEntry")
            .field("opaque_display_list_ptr", &self.opaque_display_list_ptr())
            .field(
                "translucent_display_list_ptr",
                &self.translucent_display_list_ptr(),
            )
            .finish()
    }
}
impl<'a> StructReader<'a> for SimpleMeshEntry<'a> {
    const SIZE: usize = 8;
    fn new(data: &'a [u8]) -> SimpleMeshEntry<'a> {
        SimpleMeshEntry { data }
    }
}
impl<'a> SimpleMeshEntry<'a> {
    pub fn opaque_display_list_ptr(self) -> Option<SegmentAddr> {
        match (&self.data[..]).read_u32::<BigEndian>().unwrap() {
            0 => None,
            addr => Some(SegmentAddr(addr)),
        }
    }
    pub fn translucent_display_list_ptr(self) -> Option<SegmentAddr> {
        match (&self.data[4..]).read_u32::<BigEndian>().unwrap() {
            0 => None,
            addr => Some(SegmentAddr(addr)),
        }
    }

    pub fn opaque_display_list(
        self,
        segment_ctx: &SegmentCtx<'a>,
    ) -> Result<Option<DisplayList<'a>>, SegmentResolveError> {
        self.opaque_display_list_ptr()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
    pub fn translucent_display_list(
        self,
        segment_ctx: &SegmentCtx<'a>,
    ) -> Result<Option<DisplayList<'a>>, SegmentResolveError> {
        self.translucent_display_list_ptr()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
}

#[derive(Clone, Copy)]
pub struct JfifMesh<'a> {
    _data: &'a [u8],
}
impl<'a> std::fmt::Debug for JfifMesh<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("JfifMesh").finish()
    }
}

#[derive(Clone, Copy)]
pub struct ClippedMesh<'a> {
    data: &'a [u8],
}
impl<'a> Debug for ClippedMesh<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ClippedMesh")
            .field("count", &self.count())
            .field("start", &self.start())
            .field("end", &self.end())
            .finish()
    }
}
impl<'a> ClippedMesh<'a> {
    pub fn new(data: &'a [u8]) -> ClippedMesh<'a> {
        ClippedMesh { data }
    }
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn start(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn end(self) -> SegmentAddr {
        SegmentAddr((&self.data[8..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn entries(self, segment_ctx: &SegmentCtx<'a>) -> Slice<'a, ClippedMeshEntry<'a>> {
        Slice::new(
            segment_ctx.resolve_range(self.start()..self.end()).unwrap(),
            self.count() as usize,
        )
    }
}

#[derive(Clone, Copy)]
pub struct ClippedMeshEntry<'a> {
    data: &'a [u8],
}
impl<'a> Debug for ClippedMeshEntry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ClippedMeshEntry")
            .field("x_range", &self.x_range())
            .field("z_range", &self.z_range())
            .field("opaque_display_list_ptr", &self.opaque_display_list_ptr())
            .field(
                "translucent_display_list_ptr",
                &self.translucent_display_list_ptr(),
            )
            .finish()
    }
}
impl<'a> StructReader<'a> for ClippedMeshEntry<'a> {
    const SIZE: usize = 16;
    fn new(data: &'a [u8]) -> ClippedMeshEntry<'a> {
        ClippedMeshEntry { data }
    }
}
impl<'a> ClippedMeshEntry<'a> {
    pub fn x_max(self) -> i16 {
        (&self.data[..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_max(self) -> i16 {
        (&self.data[2..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn x_min(self) -> i16 {
        (&self.data[4..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_min(self) -> i16 {
        (&self.data[6..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn opaque_display_list_ptr(self) -> Option<SegmentAddr> {
        match (&self.data[8..]).read_u32::<BigEndian>().unwrap() {
            0 => None,
            addr => Some(SegmentAddr(addr)),
        }
    }
    pub fn translucent_display_list_ptr(self) -> Option<SegmentAddr> {
        match (&self.data[12..]).read_u32::<BigEndian>().unwrap() {
            0 => None,
            addr => Some(SegmentAddr(addr)),
        }
    }

    pub fn x_range(self) -> RangeInclusive<i16> {
        self.x_min()..=self.x_max()
    }
    pub fn z_range(self) -> RangeInclusive<i16> {
        self.z_min()..=self.z_max()
    }
    pub fn opaque_display_list(
        self,
        segment_ctx: &SegmentCtx<'a>,
    ) -> Result<Option<DisplayList<'a>>, SegmentResolveError> {
        self.opaque_display_list_ptr()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
    pub fn translucent_display_list(
        self,
        segment_ctx: &SegmentCtx<'a>,
    ) -> Result<Option<DisplayList<'a>>, SegmentResolveError> {
        self.translucent_display_list_ptr()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
}
