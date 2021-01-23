use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::type_::TypeDescriptor;

pub const MESH_TYPE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "MeshType",
    underlying: PrimitiveType::U8,
    values: &[(0x00, "SIMPLE"), (0x01, "JFIF"), (0x02, "CLIPPED")],
});

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MeshType(pub u8);

impl MeshType {
    pub const SIMPLE: MeshType = MeshType(0x00);
    pub const JFIF: MeshType = MeshType(0x01);
    pub const CLIPPED: MeshType = MeshType(0x02);

    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl<'scope> Instantiate<'scope> for MeshType {
    fn new(data: &'scope [u8]) -> Self {
        MeshType(data[0])
    }
}
