use scoped_owner::ScopedOwner;

use crate::collision::{Collision, COLLISION_PTR_DESC};
use crate::delimited::{is_end, Delimited};
use crate::fs::{LazyFileSystem, VromAddr, VROM_ADDR_DESC};
use crate::header::scene::camera::{Camera, CAMERA_DESC};
use crate::header::scene::elf_message::{ElfMessage, ELF_MESSAGE_DESC};
use crate::header::scene::global_object::{GlobalObject, GLOBAL_OBJECT_DESC};
use crate::header::scene::type_::{SceneHeaderType, SCENE_HEADER_TYPE_DESC};
use crate::header::{Actor, AlternateHeadersHeader, ACTOR_DESC, ALTERNATE_HEADERS_HEADER_DESC};
use crate::reflect::primitive::{BOOL_DESC, I16_DESC, U16_DESC, U32_DESC, U8_DESC};
use crate::reflect::sourced::RangeSourced;
use crate::room::Room;
use crate::scene::{Lighting, LIGHTING_DESC};

pub mod camera;
pub mod elf_message;
pub mod global_object;
pub mod type_;

compile_interfaces! {
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
