use oot_explorer_read::{ReadError, Slice};
use oot_explorer_reflect::{I16_DESC, SEGMENT_ADDR_DESC, U16_DESC, U32_DESC};
use oot_explorer_segment::{SegmentAddr, SegmentError, SegmentTable};
use oot_explorer_vrom::Vrom;

declare_pointer_descriptor!(Collision);

compile_interfaces! {
    #[layout(size = 0x2c, align_bits = 2)]
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

    #[layout(size = 6, align_bits = 1)]
    struct Vertex {
        i16 x @0;
        i16 y @2;
        i16 z @4;
    }

    #[layout(size = 0x10, align_bits = 1)]
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

    #[layout(size = 8, align_bits = 2)]
    struct TriangleType {
        u32 high_value @0;
        u32 low_value @4;
    }

    #[layout(size = 8, align_bits = 2)]
    struct CameraData {
        u16 s_type @0;
        u16 camera_count @2;
        SegmentAddr camera_ptr @4;
    }

    #[layout(size = 0x10, align_bits = 2)]
    struct WaterBox {
        i16 x_min @0x0;
        i16 y_surface @0x2;
        i16 z_min @0x4;
        i16 x_span @0x6;
        i16 z_span @0x8;
        u32 flags @0xc;
    }
}

impl Collision {
    pub fn infer_triangle_type_count(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<u32, ReadError> {
        let accumulate_max_triangle_type =
            |acc: u16, triangle: Result<Triangle, ReadError>| -> Result<u16, ReadError> {
                Ok(acc.max(triangle?.type_(vrom)))
            };

        Ok(self
            .triangles(vrom, segment_table)?
            .iter(vrom)
            .try_fold(0, accumulate_max_triangle_type)? as u32
            + 1)
    }

    pub fn triangle_types_with_len(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
        len: u32,
    ) -> Result<Slice<TriangleType>, SegmentError> {
        Ok(Slice::new(
            segment_table.resolve(self.triangle_types_ptr(vrom))?,
            len,
        ))
    }

    pub fn triangle_types_auto_len(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Slice<TriangleType>, ReadError> {
        Ok(self.triangle_types_with_len(
            vrom,
            segment_table,
            self.infer_triangle_type_count(vrom, segment_table)?,
        )?)
    }

    pub fn camera_data_with_len(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
        len: u32,
    ) -> Result<Slice<CameraData>, SegmentError> {
        Ok(Slice::new(
            segment_table.resolve(self.camera_data_ptr(vrom))?,
            len,
        ))
    }
}

impl Triangle {
    pub fn vertex_a(self, vrom: Vrom<'_>) -> u16 {
        self.vertex_a_and_flags(vrom) & 0x1fff
    }

    pub fn collision_flags(self, vrom: Vrom<'_>) -> u8 {
        (self.vertex_a_and_flags(vrom) >> 13) as u8
    }

    pub fn vertex_b(self, vrom: Vrom<'_>) -> u16 {
        self.vertex_b_and_flags(vrom) & 0x1fff
    }

    pub fn conveyor(self, vrom: Vrom<'_>) -> bool {
        (self.vertex_b_and_flags(vrom) & 0x2000) == 0x2000
    }

    pub fn vertex_c(self, vrom: Vrom<'_>) -> u16 {
        self.vertex_c_and_flags(vrom) & 0x1fff
    }
}
