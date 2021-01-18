use crate::header::alternate::AlternateHeadersHeader;
use crate::header::room::type_::RoomHeaderType;
use crate::header::room::variant::actor::ActorListHeader;
use crate::header::room::variant::behavior::RoomBehaviorHeader;
use crate::header::room::variant::mesh::MeshHeader;
use crate::header::room::variant::object::ObjectListHeader;
use crate::header::room::variant::skybox::RoomSkyboxHeader;
use crate::header::room::variant::sound::RoomSoundHeader;
use crate::header::room::variant::time::TimeHeader;
use crate::header::room::variant::wind::WindHeader;
use crate::header::room::variant::RoomHeaderVariant;

pub mod iter;
pub mod type_;
pub mod variant;

#[derive(Clone, Copy)]
pub struct RoomHeader<'a> {
    data: &'a [u8],
}

impl<'a> RoomHeader<'a> {
    pub const SIZE: usize = 8;

    pub fn new(data: &'a [u8]) -> RoomHeader<'a> {
        RoomHeader { data }
    }

    pub fn type_(self) -> RoomHeaderType {
        RoomHeaderType(self.data[0])
    }

    pub fn variant(self) -> RoomHeaderVariant<'a> {
        let data = self.data;
        match self.type_() {
            RoomHeaderType::ACTOR_LIST => RoomHeaderVariant::ActorList(ActorListHeader::new(data)),
            RoomHeaderType::WIND => RoomHeaderVariant::Wind(WindHeader::new(data)),
            RoomHeaderType::BEHAVIOR => RoomHeaderVariant::Behavior(RoomBehaviorHeader::new(data)),
            RoomHeaderType::MESH => RoomHeaderVariant::Mesh(MeshHeader::new(data)),
            RoomHeaderType::OBJECT_LIST => {
                RoomHeaderVariant::ObjectList(ObjectListHeader::new(data))
            }
            RoomHeaderType::TIME => RoomHeaderVariant::Time(TimeHeader::new(data)),
            RoomHeaderType::SKYBOX => RoomHeaderVariant::Skybox(RoomSkyboxHeader::new(data)),
            RoomHeaderType::END => RoomHeaderVariant::End,
            RoomHeaderType::SOUND => RoomHeaderVariant::Sound(RoomSoundHeader::new(data)),
            RoomHeaderType::ALTERNATE_HEADERS => {
                RoomHeaderVariant::AlternateHeaders(AlternateHeadersHeader::new(data))
            }
            type_ => panic!("unexpected room header type: 0x{:02x}", type_.0),
        }
    }
}
