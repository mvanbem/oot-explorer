use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::type_::TypeDescriptor;

pub const ROOM_HEADER_TYPE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "RoomHeaderType",
    underlying: PrimitiveType::U8,
    values: &[
        (0x01, "ACTOR_LIST"),
        (0x05, "WIND"),
        (0x08, "BEHAVIOR"),
        (0x0a, "MESH"),
        (0x0b, "OBJECT_LIST"),
        (0x10, "TIME"),
        (0x12, "SKYBOX"),
        (0x14, "END"),
        (0x16, "SOUND"),
        (0x18, "ALTERNATE_HEADERS"),
    ],
});

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RoomHeaderType(pub u8);

impl RoomHeaderType {
    pub const ACTOR_LIST: RoomHeaderType = RoomHeaderType(0x01);
    pub const WIND: RoomHeaderType = RoomHeaderType(0x05);
    pub const BEHAVIOR: RoomHeaderType = RoomHeaderType(0x08);
    pub const MESH: RoomHeaderType = RoomHeaderType(0x0a);
    pub const OBJECT_LIST: RoomHeaderType = RoomHeaderType(0x0b);
    pub const TIME: RoomHeaderType = RoomHeaderType(0x10);
    pub const SKYBOX: RoomHeaderType = RoomHeaderType(0x12);
    pub const END: RoomHeaderType = RoomHeaderType(0x14);
    pub const SOUND: RoomHeaderType = RoomHeaderType(0x16);
    pub const ALTERNATE_HEADERS: RoomHeaderType = RoomHeaderType(0x18);

    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl<'scope> Instantiate<'scope> for RoomHeaderType {
    fn new(data: &'scope [u8]) -> Self {
        RoomHeaderType(data[0])
    }
}
