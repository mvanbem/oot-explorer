use crate::gbi::DisplayList;
use crate::mesh::type_::{MeshType, MESH_TYPE_DESC};
use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::{PrimitiveType, I16_DESC, I8_DESC, U8_DESC};
use crate::reflect::type_::TypeDescriptor;
use crate::segment::{SegmentAddr, SegmentCtx, SegmentResolveError, SEGMENT_ADDR_DESC};
use crate::slice::Slice;

pub mod type_;

declare_pointer_descriptor!(Mesh);
declare_pointer_descriptor!(SimpleMeshEntry);

compile_interfaces! {
    #[size(0x10)]
    union Mesh: MeshType @0 {
        struct SimpleMesh simple #MeshType::SIMPLE;
        struct JfifMesh jfif #MeshType::JFIF;
        struct ClippedMesh clipped #MeshType::CLIPPED;
    }

    #[size(0xc)]
    struct SimpleMesh {
        u8 count @1;
        SegmentAddr start @4;
        SegmentAddr end @8;
    }

    #[size(8)]
    struct SimpleMeshEntry {
        // TODO: Guarded getters in codegen, then type these as pointers.
        SegmentAddr opaque_display_list_ptr @0;
        SegmentAddr translucent_display_list_ptr @4;
    }

    #[size(0x10)]
    union JfifMesh: JfifMeshType @1 {
        struct SingleJfif single #JfifMeshType::SINGLE;
        struct MultipleJfif multiple #JfifMeshType::MULTIPLE;
    }

    #[size(0x20)]
    struct SingleJfif {
        struct SimpleMeshEntry* mesh_entry @4;
        struct Background background @8;
    }

    #[size(0x10)]
    struct MultipleJfif {
        struct SimpleMeshEntry* mesh_entry @4;
        struct MultipleJfifEntry[u8 @8]* background_entries @0x0c;
    }

    #[size(0x1c)]
    struct MultipleJfifEntry {
        i8 id @2;
        struct Background background @4;
    }

    #[size(0x18)]
    struct Background {
        SegmentAddr ptr @0;
    }

    #[size(0xc)]
    struct ClippedMesh {
        u8 count @1;
        SegmentAddr start @4;
        SegmentAddr end @8;
    }

    #[size(0x10)]
    struct ClippedMeshEntry {
        i16 x_max @0;
        i16 z_max @2;
        i16 x_min @4;
        i16 z_min @6;
        SegmentAddr opaque_display_list_ptr @8;
        SegmentAddr translucent_display_list_ptr @0xc;
    }
}

impl<'scope> SimpleMesh<'scope> {
    pub fn entries(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Slice<'scope, SimpleMeshEntry<'scope>> {
        Slice::new(
            segment_ctx.resolve_range(self.start()..self.end()).unwrap(),
            self.count() as usize,
        )
    }
}

impl<'scope> SimpleMeshEntry<'scope> {
    // TODO: Guarded getters in codegen.

    pub fn opaque_display_list(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Result<Option<DisplayList<'scope>>, SegmentResolveError> {
        self.opaque_display_list_ptr()
            .non_null()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }

    pub fn translucent_display_list(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Result<Option<DisplayList<'scope>>, SegmentResolveError> {
        self.translucent_display_list_ptr()
            .non_null()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
}

pub const JFIF_MESH_TYPE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "JfifMeshType",
    underlying: PrimitiveType::U8,
    values: &[(0x01, "SINGLE"), (0x02, "MULTIPLE")],
});

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct JfifMeshType(pub u8);

impl JfifMeshType {
    pub const SINGLE: JfifMeshType = JfifMeshType(0x01);
    pub const MULTIPLE: JfifMeshType = JfifMeshType(0x02);

    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl<'scope> Instantiate<'scope> for JfifMeshType {
    fn new(data: &'scope [u8]) -> Self {
        JfifMeshType(data[0])
    }
}

impl<'scope> ClippedMesh<'scope> {
    pub fn entries(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Slice<'scope, ClippedMeshEntry<'scope>> {
        Slice::new(
            segment_ctx.resolve_range(self.start()..self.end()).unwrap(),
            self.count() as usize,
        )
    }
}

impl<'scope> ClippedMeshEntry<'scope> {
    pub fn opaque_display_list(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Result<Option<DisplayList<'scope>>, SegmentResolveError> {
        self.opaque_display_list_ptr()
            .non_null()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }

    pub fn translucent_display_list(
        self,
        segment_ctx: &SegmentCtx<'scope>,
    ) -> Result<Option<DisplayList<'scope>>, SegmentResolveError> {
        self.translucent_display_list_ptr()
            .non_null()
            .map(|ptr| segment_ctx.resolve(ptr).map(|addr| DisplayList::new(addr)))
            .transpose()
    }
}
