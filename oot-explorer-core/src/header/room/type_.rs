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
}
