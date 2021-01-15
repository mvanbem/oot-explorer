#[derive(Clone, Copy, Eq, PartialEq)]
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
