use oot_explorer_read::{is_end, Sentinel};
use oot_explorer_reflect::{BOOL_DESC, I8_DESC, U16_DESC, U8_DESC};
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::header_common::{
    Actor, AlternateHeadersHeader, ACTOR_DESC, ALTERNATE_HEADERS_HEADER_DESC,
};
use crate::mesh::{Mesh, MESH_PTR_DESC};
use crate::object::{ObjectId, OBJECT_ID_DESC};

compile_interfaces! {
    enum RoomHeaderType: u8 {
        ACTOR_LIST = 0x01;
        WIND = 0x05;
        BEHAVIOR = 0x08;
        MESH = 0x0a;
        OBJECT_LIST = 0x0b;
        TIME = 0x10;
        SKYBOX = 0x12;
        END = 0x14;
        SOUND = 0x16;
        ALTERNATE_HEADERS = 0x18;
    }

    #[is_end(is_end::<RoomHeader>)]
    #[layout(size = 8, align_bits = 2)]
    union RoomHeader: RoomHeaderType @0 {
        struct ActorListHeader actor_list #RoomHeaderType::ACTOR_LIST;
        struct WindHeader wind #RoomHeaderType::WIND;
        struct BehaviorHeader behavior #RoomHeaderType::BEHAVIOR;
        struct MeshHeader mesh #RoomHeaderType::MESH;
        struct ObjectListHeader object_list #RoomHeaderType::OBJECT_LIST;
        struct TimeHeader time #RoomHeaderType::TIME;
        struct RoomSkyboxHeader skybox #RoomHeaderType::SKYBOX;
        struct EndHeader end #RoomHeaderType::END;
        struct RoomSoundHeader sound #RoomHeaderType::SOUND;
        struct AlternateHeadersHeader alternate_headers #RoomHeaderType::ALTERNATE_HEADERS;
    }

    #[layout(size = 8, align_bits = 2)]
    struct ActorListHeader {
        struct Actor[u8 @1]* actor_list @4;
    }

    #[layout(size = 8, align_bits = 2)]
    struct WindHeader {
        i8 west @4;
        i8 up @5;
        i8 south @6;
        u8 strength @7;
    }

    #[layout(size = 8, align_bits = 2)]
    struct BehaviorHeader {
        // Affects Sun's Song, backflipping with A.
        u8 x @1;
        u8 flags @6;
        u8 idle_animation_or_heat @7;
    }

    #[layout(size = 8, align_bits = 2)]
    struct MeshHeader {
        struct Mesh* mesh @4;
    }

    #[layout(size = 8, align_bits = 2)]
    struct ObjectListHeader {
        ObjectId[u8 @1]* objects @4;
    }

    #[layout(size = 8, align_bits = 2)]
    struct TimeHeader {
        u16 raw_time_override @4;
        i8 time_speed @6;
    }

    #[layout(size = 8, align_bits = 2)]
    struct RoomSkyboxHeader {
        bool disable_sky @4;
        bool disable_sun_moon @5;
    }

    #[layout(size = 8, align_bits = 2)]
    struct EndHeader {}

    #[layout(size = 8, align_bits = 2)]
    struct RoomSoundHeader {
        u8 echo @7;
    }
}

impl Sentinel for RoomHeader {
    const ITER_YIELDS_SENTINEL_VALUE: bool = true;

    fn is_end(&self, vrom: oot_explorer_vrom::Vrom<'_>) -> bool {
        self.discriminant(vrom) == RoomHeaderType::END
    }
}

impl BehaviorHeader {
    // TODO: Codegen for bitfields.
    pub fn disable_warp_songs(self, vrom: Vrom<'_>) -> u8 {
        self.flags(vrom) >> 4
    }

    pub fn show_invisible_actors(self, vrom: Vrom<'_>) -> bool {
        self.flags(vrom) & 1 == 1
    }
}

impl TimeHeader {
    pub fn time_override(self, vrom: Vrom<'_>) -> Option<u16> {
        match self.raw_time_override(vrom) {
            0xffff => None,
            time => Some(time),
        }
    }
}
