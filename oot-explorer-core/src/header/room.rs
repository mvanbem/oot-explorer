use crate::delimited::{is_end, Delimited};
use crate::header::{Actor, AlternateHeadersHeader, ACTOR_DESC, ALTERNATE_HEADERS_HEADER_DESC};
use crate::mesh::{Mesh, MESH_PTR_DESC};
use crate::object::{ObjectId, OBJECT_ID_DESC};
use crate::reflect::primitive::{BOOL_DESC, I8_DESC, U16_DESC, U8_DESC};

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

    #[size(8)]
    #[is_end(|scope, fs, addr| is_end::<RoomHeader>(scope, fs, addr))]
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

    struct ActorListHeader {
        struct Actor[u8 @1]* actor_list @4;
    }

    struct WindHeader {
        i8 west @4;
        i8 up @5;
        i8 south @6;
        u8 strength @7;
    }

    struct BehaviorHeader {
        // Affects Sun's Song, backflipping with A.
        u8 x @1;
        u8 flags @6;
        u8 idle_animation_or_heat @7;
    }

    struct MeshHeader {
        struct Mesh* mesh @4;
    }

    struct ObjectListHeader {
        ObjectId[u8 @1]* objects @4;
    }

    struct TimeHeader {
        u16 raw_time_override @4;
        i8 time_speed @6;
    }

    struct RoomSkyboxHeader {
        bool disable_sky @4;
        bool disable_sun_moon @5;
    }

    struct EndHeader {}

    struct RoomSoundHeader {
        u8 echo @7;
    }
}

impl<'scope> Delimited for RoomHeader<'scope> {
    fn is_end(&self) -> bool {
        self.discriminant() == RoomHeaderType::END
    }
}

impl<'scope> BehaviorHeader<'scope> {
    // TODO: Codegen for bitfields.
    pub fn disable_warp_songs(self) -> u8 {
        self.flags() >> 4
    }

    pub fn show_invisible_actors(self) -> bool {
        self.flags() & 1 == 1
    }
}

impl<'scope> TimeHeader<'scope> {
    pub fn time_override(self) -> Option<u16> {
        match self.raw_time_override() {
            0xffff => None,
            time => Some(time),
        }
    }
}
