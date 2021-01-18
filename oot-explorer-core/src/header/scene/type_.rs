use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::type_::TypeDescriptor;

pub const SCENE_HEADER_TYPE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "SceneHeaderType",
    underlying: PrimitiveType::U8,
    values: &[
        /* 0x00 */ Some("START_POSITIONS"),
        /* 0x01 */ None,
        /* 0x02 */ None,
        /* 0x03 */ Some("COLLISION"),
        /* 0x04 */ Some("ROOM_LIST"),
        /* 0x05 */ None,
        /* 0x06 */ Some("ENTRANCE_LIST"),
        /* 0x07 */ Some("SPECIAL_OBJECTS"),
        /* 0x08 */ None,
        /* 0x09 */ None,
        /* 0x0a */ None,
        /* 0x0b */ None,
        /* 0x0c */ None,
        /* 0x0d */ Some("PATHWAYS"),
        /* 0x0e */ Some("TRANSITION_ACTORS"),
        /* 0x0f */ Some("LIGHTING"),
        /* 0x10 */ None,
        /* 0x11 */ Some("SKYBOX"),
        /* 0x12 */ None,
        /* 0x13 */ Some("EXIT_LIST"),
        /* 0x14 */ Some("END"),
        /* 0x15 */ Some("SOUND"),
        /* 0x16 */ None,
        /* 0x17 */ None,
        /* 0x18 */ Some("ALTERNATE_HEADERS"),
        /* 0x19 */ Some("CAMERA_AND_WORLD_MAP"),
    ],
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneHeaderType(pub u8);

impl SceneHeaderType {
    pub const START_POSITIONS: SceneHeaderType = SceneHeaderType(0x00);
    pub const COLLISION: SceneHeaderType = SceneHeaderType(0x03);
    pub const ROOM_LIST: SceneHeaderType = SceneHeaderType(0x04);
    pub const ENTRANCE_LIST: SceneHeaderType = SceneHeaderType(0x06);
    pub const SPECIAL_OBJECTS: SceneHeaderType = SceneHeaderType(0x07);
    pub const PATHWAYS: SceneHeaderType = SceneHeaderType(0x0d);
    pub const TRANSITION_ACTORS: SceneHeaderType = SceneHeaderType(0x0e);
    pub const LIGHTING: SceneHeaderType = SceneHeaderType(0x0f);
    pub const SKYBOX: SceneHeaderType = SceneHeaderType(0x11);
    pub const EXIT_LIST: SceneHeaderType = SceneHeaderType(0x13);
    pub const END: SceneHeaderType = SceneHeaderType(0x14);
    pub const SOUND: SceneHeaderType = SceneHeaderType(0x15);
    pub const ALTERNATE_HEADERS: SceneHeaderType = SceneHeaderType(0x18);
    pub const CAMERA_AND_WORLD_MAP: SceneHeaderType = SceneHeaderType(0x19);
}
