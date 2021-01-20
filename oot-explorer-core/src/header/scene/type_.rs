use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::type_::TypeDescriptor;

pub const SCENE_HEADER_TYPE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "SceneHeaderType",
    underlying: PrimitiveType::U8,
    values: &[
        (0x00, "START_POSITIONS"),
        (0x03, "COLLISION"),
        (0x04, "ROOM_LIST"),
        (0x06, "ENTRANCE_LIST"),
        (0x07, "SPECIAL_OBJECTS"),
        (0x0d, "PATHWAYS"),
        (0x0e, "TRANSITION_ACTORS"),
        (0x0f, "LIGHTING"),
        (0x11, "SKYBOX"),
        (0x13, "EXIT_LIST"),
        (0x14, "END"),
        (0x15, "SOUND"),
        (0x18, "ALTERNATE_HEADERS"),
        (0x19, "CAMERA_AND_WORLD_MAP"),
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

    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl<'scope> Instantiate<'scope> for SceneHeaderType {
    fn new(data: &'scope [u8]) -> Self {
        SceneHeaderType(data[0])
    }
}
