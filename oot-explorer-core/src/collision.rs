use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::{Slice, StructReader};
use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};
use std::ops::RangeInclusive;

#[derive(Clone, Copy)]
pub struct Collision<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Collision<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Collision")
            .field("bounds", &self.bounds())
            .field("vertex_count", &self.vertex_count())
            .field("vertex_ptr", &self.vertex_ptr())
            .field("triangle_count", &self.triangle_count())
            .field("triangle_ptr", &self.triangle_ptr())
            .field("triangle_type_ptr", &self.triangle_type_ptr())
            .field("camera_data_ptr", &self.camera_data_ptr())
            .field("water_box_count", &self.water_box_count())
            .field("water_box_ptr", &self.water_box_ptr())
            .finish()
    }
}
impl<'a> Collision<'a> {
    pub const SIZE: usize = 0x2c;

    pub fn new(data: &'a [u8]) -> Collision<'a> {
        Collision { data }
    }
    pub fn x_min(self) -> i16 {
        (&self.data[0x00..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn y_min(self) -> i16 {
        (&self.data[0x02..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_min(self) -> i16 {
        (&self.data[0x04..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn x_max(self) -> i16 {
        (&self.data[0x06..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn y_max(self) -> i16 {
        (&self.data[0x08..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_max(self) -> i16 {
        (&self.data[0x0a..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn vertex_count(self) -> u16 {
        (&self.data[0x0c..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn vertex_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x10..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn triangle_count(self) -> u16 {
        (&self.data[0x14..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn triangle_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x18..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn triangle_type_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x1c..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn camera_data_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x20..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn water_box_count(self) -> u16 {
        (&self.data[0x24..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn water_box_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[0x28..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn bounds(self) -> RangeInclusive<[i16; 3]> {
        [self.x_min(), self.y_min(), self.z_min()]..=[self.x_max(), self.y_max(), self.z_max()]
    }
    pub fn vertices(self, ctx: &SegmentCtx<'a>) -> Slice<'a, Vertex<'a>> {
        Slice::new(
            ctx.resolve(self.vertex_ptr()).unwrap(),
            self.vertex_count() as usize,
        )
    }
    pub fn triangles(self, ctx: &SegmentCtx<'a>) -> Slice<'a, Triangle<'a>> {
        Slice::new(
            ctx.resolve(self.triangle_ptr()).unwrap(),
            self.triangle_count() as usize,
        )
    }
    pub fn infer_triangle_type_count(self, ctx: &SegmentCtx<'a>) -> usize {
        self.triangles(ctx)
            .iter()
            .map(|triangle| triangle.type_())
            .max()
            .map(|highest| highest as usize + 1)
            .unwrap_or(0)
    }
    pub fn triangle_types_with_len(
        self,
        ctx: &SegmentCtx<'a>,
        len: usize,
    ) -> Slice<'a, TriangleType<'a>> {
        Slice::new(ctx.resolve(self.triangle_type_ptr()).unwrap(), len)
    }
    pub fn triangle_types_auto_len(self, ctx: &SegmentCtx<'a>) -> Slice<'a, TriangleType<'a>> {
        self.triangle_types_with_len(ctx, self.infer_triangle_type_count(ctx))
    }
    pub fn camera_data_with_len(
        self,
        ctx: &SegmentCtx<'a>,
        len: usize,
    ) -> Slice<'a, CameraData<'a>> {
        Slice::new(ctx.resolve(self.camera_data_ptr()).unwrap(), len)
    }
    pub fn water_boxes(self, ctx: &SegmentCtx<'a>) -> Slice<'a, WaterBox<'a>> {
        Slice::new(
            ctx.resolve(self.water_box_ptr()).unwrap(),
            self.water_box_count() as usize,
        )
    }
}

#[derive(Clone, Copy)]
pub struct Vertex<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Vertex<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::Vertex")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .finish()
    }
}
impl<'a> StructReader<'a> for Vertex<'a> {
    const SIZE: usize = 6;
    fn new(data: &'a [u8]) -> Vertex<'a> {
        Vertex { data }
    }
}
impl<'a> Vertex<'a> {
    pub fn x(self) -> i16 {
        (&self.data[..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn y(self) -> i16 {
        (&self.data[2..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z(self) -> i16 {
        (&self.data[4..]).read_i16::<BigEndian>().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct Triangle<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Triangle<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::Triangle")
            .field("type_", &self.type_())
            .field(
                "vertices",
                &[self.vertex_a(), self.vertex_b(), self.vertex_c()],
            )
            .field("collision_flags", &self.collision_flags())
            .field("conveyor", &self.conveyor())
            .field(
                "plane",
                &[
                    self.plane_a(),
                    self.plane_b(),
                    self.plane_c(),
                    self.plane_d(),
                ],
            )
            .finish()
    }
}
impl<'a> StructReader<'a> for Triangle<'a> {
    const SIZE: usize = 0x10;
    fn new(data: &'a [u8]) -> Triangle<'a> {
        Triangle { data }
    }
}
impl<'a> Triangle<'a> {
    pub fn type_(self) -> u16 {
        (&self.data[0x0..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn vertex_a_and_flags(self) -> u16 {
        (&self.data[0x2..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn vertex_b_and_flags(self) -> u16 {
        (&self.data[0x4..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn vertex_c_and_flags(self) -> u16 {
        (&self.data[0x6..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn plane_a(self) -> i16 {
        (&self.data[0x8..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn plane_b(self) -> i16 {
        (&self.data[0xa..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn plane_c(self) -> i16 {
        (&self.data[0xc..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn plane_d(self) -> i16 {
        (&self.data[0xe..]).read_i16::<BigEndian>().unwrap()
    }

    pub fn vertex_a(self) -> u16 {
        self.vertex_a_and_flags() & 0x1fff
    }
    pub fn collision_flags(self) -> u8 {
        (self.vertex_a_and_flags() >> 13) as u8
    }
    pub fn vertex_b(self) -> u16 {
        self.vertex_b_and_flags() & 0x1fff
    }
    pub fn conveyor(self) -> bool {
        (self.vertex_b_and_flags() & 0x2000) == 0x2000
    }
    pub fn vertex_c(self) -> u16 {
        self.vertex_c_and_flags() & 0x1fff
    }
}

#[derive(Clone, Copy)]
pub struct TriangleType<'a> {
    data: &'a [u8],
}
impl<'a> Debug for TriangleType<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::TriangleType")
            .field("high_value", &self.high_value())
            .field("low_value", &self.low_value())
            .finish()
    }
}
impl<'a> StructReader<'a> for TriangleType<'a> {
    const SIZE: usize = 8;
    fn new(data: &'a [u8]) -> TriangleType<'a> {
        TriangleType { data }
    }
}
impl<'a> TriangleType<'a> {
    pub fn high_value(self) -> u32 {
        (&self.data[..]).read_u32::<BigEndian>().unwrap()
    }
    pub fn low_value(self) -> u32 {
        (&self.data[4..]).read_u32::<BigEndian>().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct CameraData<'a> {
    data: &'a [u8],
}
impl<'a> Debug for CameraData<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::CameraData")
            .field("s_type", &self.s_type())
            .field("camera_count", &self.camera_count())
            .field("camera_ptr", &self.camera_ptr())
            .finish()
    }
}
impl<'a> StructReader<'a> for CameraData<'a> {
    const SIZE: usize = 8;
    fn new(data: &'a [u8]) -> CameraData<'a> {
        CameraData { data }
    }
}
impl<'a> CameraData<'a> {
    pub fn s_type(self) -> u16 {
        (&self.data[..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn camera_count(self) -> u16 {
        (&self.data[2..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn camera_ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }
}

#[derive(Clone, Copy)]
pub struct WaterBox<'a> {
    data: &'a [u8],
}
impl<'a> Debug for WaterBox<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::WaterBox")
            .field("x_min", &self.x_min())
            .field("y_surface", &self.y_surface())
            .field("z_min", &self.z_min())
            .field("x_span", &self.x_span())
            .field("z_span", &self.z_span())
            .field("flags", &self.flags())
            .finish()
    }
}
impl<'a> StructReader<'a> for WaterBox<'a> {
    const SIZE: usize = 0x10;
    fn new(data: &'a [u8]) -> WaterBox<'a> {
        WaterBox { data }
    }
}
impl<'a> WaterBox<'a> {
    pub fn x_min(self) -> i16 {
        (&self.data[0x0..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn y_surface(self) -> i16 {
        (&self.data[0x2..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_min(self) -> i16 {
        (&self.data[0x4..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn x_span(self) -> i16 {
        (&self.data[0x6..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn z_span(self) -> i16 {
        (&self.data[0x8..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn flags(self) -> u32 {
        (&self.data[0xc..]).read_u32::<BigEndian>().unwrap()
    }
}
