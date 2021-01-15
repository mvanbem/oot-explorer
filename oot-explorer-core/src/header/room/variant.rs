use crate::header::alternate::AlternateHeadersHeader;

pub mod actor;
pub mod behavior;
pub mod mesh;
pub mod object;
pub mod skybox;
pub mod sound;
pub mod time;
pub mod wind;

use actor::ActorListHeader;
use behavior::RoomBehaviorHeader;
use mesh::MeshHeader;
use object::ObjectListHeader;
use skybox::RoomSkyboxHeader;
use sound::RoomSoundHeader;
use time::TimeHeader;
use wind::WindHeader;

#[derive(Clone, Copy, Debug)]
pub enum RoomHeaderVariant<'a> {
    ActorList(ActorListHeader<'a>),
    Wind(WindHeader<'a>),
    Behavior(RoomBehaviorHeader<'a>),
    Mesh(MeshHeader<'a>),
    ObjectList(ObjectListHeader<'a>),
    Time(TimeHeader<'a>),
    Skybox(RoomSkyboxHeader<'a>),
    End,
    Sound(RoomSoundHeader<'a>),
    AlternateHeaders(AlternateHeadersHeader<'a>),
}
