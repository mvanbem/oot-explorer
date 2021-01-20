use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};
use std::ops::RangeInclusive;

use crate::gbi::DisplayList;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::sized::ReflectSized;
use crate::segment::{SegmentAddr, SegmentCtx, SegmentResolveError};
use crate::slice::Slice;

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
            0x01 => MeshVariant::Jfif(JfifMesh { data }),
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
impl<'a> Debug for SimpleMesh<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
impl<'a> Debug for SimpleMeshEntry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SimpleMeshEntry")
            .field("opaque_display_list_ptr", &self.opaque_display_list_ptr())
            .field(
                "translucent_display_list_ptr",
                &self.translucent_display_list_ptr(),
            )
            .finish()
    }
}
impl<'a> Instantiate<'a> for SimpleMeshEntry<'a> {
    fn new(data: &'a [u8]) -> SimpleMeshEntry<'a> {
        SimpleMeshEntry { data }
    }
}
impl<'a> ReflectSized for SimpleMeshEntry<'a> {
    const SIZE: usize = 8;
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
    data: &'a [u8],
}

impl<'a> JfifMesh<'a> {
    pub fn data(self) -> &'a [u8] {
        self.data
    }

    pub fn format(self) -> u8 {
        self.data[1]
    }

    pub fn variant(self) -> JfifMeshVariant<'a> {
        let data = self.data;
        match self.format() {
            0x01 => JfifMeshVariant::Single(SingleJfif { data }),
            0x02 => JfifMeshVariant::Multiple(MultipleJfif { data }),
            format => panic!("unexpected JFIF format: 0x{:02x}", format),
        }
    }
}

impl<'a> Debug for JfifMesh<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("JfifMesh").finish()
    }
}

#[derive(Clone, Copy)]
pub enum JfifMeshVariant<'a> {
    Single(SingleJfif<'a>),
    Multiple(MultipleJfif<'a>),
}

#[derive(Clone, Copy)]
pub struct SingleJfif<'a> {
    data: &'a [u8],
}

impl<'a> SingleJfif<'a> {
    pub fn mesh_entry_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn mesh_entry(self, segment_ctx: &SegmentCtx<'a>) -> SimpleMeshEntry<'a> {
        SimpleMeshEntry {
            data: segment_ctx.resolve(self.mesh_entry_ptr()).unwrap(),
        }
    }

    pub fn background(self) -> Background<'a> {
        Background {
            data: &self.data[8..],
        }
    }
}

#[derive(Clone, Copy)]
pub struct MultipleJfif<'a> {
    data: &'a [u8],
}

impl<'a> MultipleJfif<'a> {
    pub fn mesh_entries_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn mesh_entries(self, segment_ctx: &SegmentCtx<'a>) -> Slice<'a, SimpleMeshEntry<'a>> {
        Slice::new(
            segment_ctx.resolve(self.mesh_entries_ptr()).unwrap(),
            self.count() as usize,
        )
    }

    pub fn count(self) -> u8 {
        self.data[8]
    }

    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x0c..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn background_entries(
        self,
        segment_ctx: &SegmentCtx<'a>,
    ) -> Slice<'a, MultipleJfifEntry<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

#[derive(Clone, Copy)]
pub struct MultipleJfifEntry<'a> {
    data: &'a [u8],
}

impl<'a> MultipleJfifEntry<'a> {
    pub fn id(self) -> i8 {
        self.data[2] as i8
    }

    pub fn background(self) -> Background<'a> {
        Background {
            data: &self.data[4..],
        }
    }
}

impl<'a> Instantiate<'a> for MultipleJfifEntry<'a> {
    fn new(data: &'a [u8]) -> MultipleJfifEntry<'a> {
        MultipleJfifEntry { data }
    }
}

impl<'a> ReflectSized for MultipleJfifEntry<'a> {
    const SIZE: usize = 0x1c;
}

#[derive(Clone, Copy)]
pub struct Background<'a> {
    data: &'a [u8],
}

impl<'a> Background<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[..]).read_u32::<BigEndian>().unwrap())
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
impl<'a> Instantiate<'a> for ClippedMeshEntry<'a> {
    fn new(data: &'a [u8]) -> ClippedMeshEntry<'a> {
        ClippedMeshEntry { data }
    }
}
impl<'a> ReflectSized for ClippedMeshEntry<'a> {
    const SIZE: usize = 16;
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
