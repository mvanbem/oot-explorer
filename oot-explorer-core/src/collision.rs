use crate::reflect::primitive::{I16_DESC, U16_DESC, U32_DESC};
use crate::segment::{SegmentAddr, SegmentCtx, SEGMENT_ADDR_DESC};
use crate::slice::Slice;

declare_pointer_descriptor!(Collision);

compile_interfaces! {
    #[size(0x2c)]
    struct Collision {
        i16 x_min @0x00;
        i16 y_min @0x02;
        i16 z_min @0x04;
        i16 x_max @0x06;
        i16 y_max @0x08;
        i16 z_max @0x0a;
        struct Vertex[u16 @0x0c]* vertices @0x10;
        struct Triangle[u16 @0x14]* triangles @0x18;
        // TODO: Type as TriangleType[?]*
        SegmentAddr triangle_types_ptr @0x1c;
        // TODO: Type as ...
        SegmentAddr camera_data_ptr @0x20;
        struct WaterBox[u16 @0x24]* water_boxes @0x28;
    }

    #[size(6)]
    struct Vertex {
        i16 x @0;
        i16 y @2;
        i16 z @4;
    }

    #[size(0x10)]
    struct Triangle {
        u16 type_ @0x0;
        u16 vertex_a_and_flags @0x2;
        u16 vertex_b_and_flags @0x4;
        u16 vertex_c_and_flags @0x6;
        i16 plane_a @0x8;
        i16 plane_b @0xa;
        i16 plane_c @0xc;
        i16 plane_d @0xe;
    }

    #[size(8)]
    struct TriangleType {
        u32 high_value @0;
        u32 low_value @4;
    }

    #[size(8)]
    struct CameraData {
        u16 s_type @0;
        u16 camera_count @2;
        SegmentAddr camera_ptr @4;
    }

    #[size(0x10)]
    struct WaterBox {
        i16 x_min @0x0;
        i16 y_surface @0x2;
        i16 z_min @0x4;
        i16 x_span @0x6;
        i16 z_span @0x8;
        u32 flags @0xc;
    }
}

impl<'scope> Collision<'scope> {
    pub fn infer_triangle_type_count(self, ctx: &SegmentCtx<'scope>) -> usize {
        self.triangles(ctx)
            .iter()
            .map(|triangle| triangle.type_())
            .max()
            .map(|highest| highest as usize + 1)
            .unwrap_or(0)
    }

    pub fn triangle_types_with_len(
        self,
        ctx: &SegmentCtx<'scope>,
        len: usize,
    ) -> Slice<'scope, TriangleType<'scope>> {
        Slice::new(ctx.resolve(self.triangle_types_ptr()).unwrap(), len)
    }

    pub fn triangle_types_auto_len(
        self,
        ctx: &SegmentCtx<'scope>,
    ) -> Slice<'scope, TriangleType<'scope>> {
        self.triangle_types_with_len(ctx, self.infer_triangle_type_count(ctx))
    }

    pub fn camera_data_with_len(
        self,
        ctx: &SegmentCtx<'scope>,
        len: usize,
    ) -> Slice<'scope, CameraData<'scope>> {
        Slice::new(ctx.resolve(self.camera_data_ptr()).unwrap(), len)
    }
}

impl<'a> Triangle<'a> {
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
