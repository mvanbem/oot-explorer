use scoped_owner::ScopedOwner;

use crate::collision::{Collision, COLLISION_PTR_DESC};
use crate::delimited::{is_end, Delimited};
use crate::fs::{LazyFileSystem, VromAddr, VROM_ADDR_DESC};
use crate::header::{Actor, AlternateHeadersHeader, ACTOR_DESC, ALTERNATE_HEADERS_HEADER_DESC};
use crate::reflect::primitive::{BOOL_DESC, I16_DESC, U16_DESC, U32_DESC, U8_DESC};
use crate::reflect::sourced::RangeSourced;
use crate::room::Room;
use crate::scene::{Lighting, LIGHTING_DESC};

compile_interfaces! {
    enum SceneHeaderType: u8 {
        START_POSITIONS = 0x00;
        COLLISION = 0x03;
        ROOM_LIST = 0x04;
        ENTRANCE_LIST = 0x06;
        SPECIAL_OBJECTS = 0x07;
        PATHWAYS = 0x0d;
        TRANSITION_ACTORS = 0x0e;
        LIGHTING = 0x0f;
        SKYBOX = 0x11;
        EXIT_LIST = 0x13;
        END = 0x14;
        SOUND = 0x15;
        ALTERNATE_HEADERS = 0x18;
        CAMERA_AND_WORLD_MAP = 0x19;
    }

    #[size(8)]
    #[is_end(|scope, fs, addr| is_end::<SceneHeader>(scope, fs, addr))]
    union SceneHeader: SceneHeaderType @0 {
        struct StartPositionsHeader start_positions #SceneHeaderType::START_POSITIONS;
        struct CollisionHeader collision #SceneHeaderType::COLLISION;
        struct RoomListHeader room_list #SceneHeaderType::ROOM_LIST;
        struct EntranceListHeader entrance_list #SceneHeaderType::ENTRANCE_LIST;
        struct SpecialObjectsHeader special_objects #SceneHeaderType::SPECIAL_OBJECTS;
        struct PathwaysHeader pathways #SceneHeaderType::PATHWAYS;
        struct TransitionActorsHeader transition_actors #SceneHeaderType::TRANSITION_ACTORS;
        struct LightingHeader lighting #SceneHeaderType::LIGHTING;
        struct SceneSkyboxHeader skybox #SceneHeaderType::SKYBOX;
        struct ExitListHeader exit_list #SceneHeaderType::EXIT_LIST;
        struct EndHeader end #SceneHeaderType::END;
        struct SceneSoundHeader sound #SceneHeaderType::SOUND;
        struct AlternateHeadersHeader alternate_headers #SceneHeaderType::ALTERNATE_HEADERS;
        struct CameraAndWorldMapHeader camera_and_world_map #SceneHeaderType::CAMERA_AND_WORLD_MAP;
    }

    struct StartPositionsHeader {
        struct Actor[u8 @1]* start_positions @4;
    }

    struct CollisionHeader {
        struct Collision* ptr @4;
    }

    struct RoomListHeader {
        struct RoomListEntry[u8 @1]* room_list @4;
    }

    #[size(0x8)]
    struct RoomListEntry {
        VromAddr start @0;
        VromAddr end @4;
    }

    struct EntranceListHeader {
        // TODO: Type this as Entrance[?]*.
        u32 ptr @4;
    }

    struct Entrance {
        u8 start_position @0;
        u8 room @1;
    }

    struct SpecialObjectsHeader {
        ElfMessage elf_message @1;
        GlobalObject global_object @6;
    }

    enum ElfMessage: u8 {
        NONE = 0x00;
        FIELD = 0x01;
        YDAN = 0x02;
    }

    enum GlobalObject: u16 {
        NONE = 0x0000;
        FIELD = 0x0002;
        DANGEON = 0x0003;
    }

    struct PathwaysHeader {
        // TODO: Type this.
        u32 ptr @4;
    }

    struct TransitionActorsHeader {
        struct TransitionActor[u8 @1]* transition_actors @4;
    }

    #[size(0x10)]
    struct TransitionActor {
        u8 room_from_front @0;
        u8 camera_from_front @1;
        u8 room_from_back @2;
        u8 camera_from_back @3;
        u16 actor_number @4;
        i16 pos_x @6;
        i16 pos_y @8;
        i16 pos_z @0xa;
        i16 rot_y @0xc;
        u16 init @0xe;
    }

    struct LightingHeader {
        struct Lighting[u8 @1]* lighting @4;
    }

    struct SceneSkyboxHeader {
        u8 skybox @4;
        bool cloudy @5;
        bool indoor_lighting @6;
    }

    struct ExitListHeader {
        // TODO: Type this as Exit[?]*.
        u32 ptr @4;
    }

    struct Exit {
        u16 value @0;
    }

    struct EndHeader {}

    struct SceneSoundHeader {
        u8 settings @1;
        u8 night_sound @6;
        u8 music @7;
    }

    struct CameraAndWorldMapHeader {
        Camera camera @1;
        u8 world_map_location @7;
    }

    enum Camera: u8 {
        FREE = 0x00;
        FIXED_WITH_ALTERNATE = 0x10;
        ROTATE_WITH_ALTERNATE = 0x20;
        FIXED = 0x30;
        ROTATE = 0x40;
        SHOOTING_GALLERY = 0x50;
    }
}

impl<'scope> Delimited for SceneHeader<'scope> {
    fn is_end(&self) -> bool {
        self.discriminant() == SceneHeaderType::END
    }
}

impl<'scope> RoomListEntry<'scope> {
    pub fn room(
        self,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
    ) -> RangeSourced<Room<'scope>> {
        let vrom_range = self.start()..self.end();
        RangeSourced::new(
            vrom_range.clone(),
            Room::new(fs.get_virtual_slice_or_die(scope, vrom_range)),
        )
    }
}
