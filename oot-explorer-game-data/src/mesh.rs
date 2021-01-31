use oot_explorer_read::{FromVrom, ReadError, Slice};
use oot_explorer_reflect::{I16_DESC, I8_DESC, SEGMENT_ADDR_DESC, U8_DESC};
use oot_explorer_segment::{SegmentAddr, SegmentError, SegmentTable};
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::gbi::DisplayList;

declare_pointer_descriptor!(Mesh);
declare_pointer_descriptor!(SimpleMeshEntry);

compile_interfaces! {
    enum MeshType: u8 {
        SIMPLE = 0x00;
        JFIF = 0x01;
        CLIPPED = 0x02;
    }

    #[layout(size = 0x10, align_bits = 2)]
    union Mesh: MeshType @0 {
        struct SimpleMesh simple #MeshType::SIMPLE;
        struct JfifMesh jfif #MeshType::JFIF;
        struct ClippedMesh clipped #MeshType::CLIPPED;
    }

    #[layout(size = 0xc, align_bits = 2)]
    struct SimpleMesh {
        u8 count @1;
        SegmentAddr start @4;
        SegmentAddr end @8;
    }

    #[layout(size = 8, align_bits = 2)]
    struct SimpleMeshEntry {
        // TODO: Guarded getters in codegen, then type these as pointers.
        SegmentAddr opaque_display_list_ptr @0;
        SegmentAddr translucent_display_list_ptr @4;
    }

    enum JfifMeshType: u8 {
        SINGLE = 0x01;
        MULTIPLE = 0x02;
    }

    #[layout(size = 0x10, align_bits = 2)]
    union JfifMesh: JfifMeshType @1 {
        struct SingleJfif single #JfifMeshType::SINGLE;
        struct MultipleJfif multiple #JfifMeshType::MULTIPLE;
    }

    #[layout(size = 0x20, align_bits = 2)]
    struct SingleJfif {
        struct SimpleMeshEntry* mesh_entry @4;
        struct Background background @8;
    }

    #[layout(size = 0x10, align_bits = 2)]
    struct MultipleJfif {
        struct SimpleMeshEntry* mesh_entry @4;
        struct MultipleJfifEntry[u8 @8]* background_entries @0x0c;
    }

    #[layout(size = 0x1c, align_bits = 2)]
    struct MultipleJfifEntry {
        i8 id @2;
        struct Background background @4;
    }

    #[layout(size = 0x18, align_bits = 2)]
    struct Background {
        SegmentAddr ptr @0;
    }

    #[layout(size = 0xc, align_bits = 2)]
    struct ClippedMesh {
        u8 count @1;
        SegmentAddr start @4;
        SegmentAddr end @8;
    }

    #[layout(size = 0x10, align_bits = 2)]
    struct ClippedMeshEntry {
        i16 x_max @0;
        i16 z_max @2;
        i16 x_min @4;
        i16 z_min @6;
        SegmentAddr opaque_display_list_ptr @8;
        SegmentAddr translucent_display_list_ptr @0xc;
    }
}

impl SimpleMesh {
    pub fn entries(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Slice<SimpleMeshEntry>, SegmentError> {
        Ok(Slice::new(
            segment_table.resolve(self.start(vrom))?,
            self.count(vrom) as u32,
        ))
    }
}

pub trait MeshEntry {
    fn opaque_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError>;
    fn translucent_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError>;
}

impl MeshEntry for SimpleMeshEntry {
    // TODO: Guarded getters in codegen.

    fn opaque_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError> {
        self.opaque_display_list_ptr(vrom)
            .non_null()
            .map(|segment_addr| DisplayList::from_vrom(vrom, segment_table.resolve(segment_addr)?))
            .transpose()
    }

    fn translucent_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError> {
        self.translucent_display_list_ptr(vrom)
            .non_null()
            .map(|segment_addr| DisplayList::from_vrom(vrom, segment_table.resolve(segment_addr)?))
            .transpose()
    }
}

impl ClippedMesh {
    pub fn entries(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Slice<ClippedMeshEntry>, SegmentError> {
        Ok(Slice::new(
            segment_table.resolve(self.start(vrom))?,
            self.count(vrom) as u32,
        ))
    }
}

impl MeshEntry for ClippedMeshEntry {
    fn opaque_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError> {
        self.opaque_display_list_ptr(vrom)
            .non_null()
            .map(|segment_addr| DisplayList::from_vrom(vrom, segment_table.resolve(segment_addr)?))
            .transpose()
    }

    fn translucent_display_list(
        self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
    ) -> Result<Option<DisplayList>, ReadError> {
        self.translucent_display_list_ptr(vrom)
            .non_null()
            .map(|segment_addr| DisplayList::from_vrom(vrom, segment_table.resolve(segment_addr)?))
            .transpose()
    }
}
