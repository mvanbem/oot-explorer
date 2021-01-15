use std::borrow::Cow;

use crate::fs::VromAddr;
use crate::header::alternate::AlternateHeadersHeader;
use crate::header::scene::variant::camera::CameraAndWorldMapHeader;
use crate::header::scene::variant::collision::CollisionHeader;
use crate::header::scene::variant::entrance::EntranceListHeader;
use crate::header::scene::variant::exit::ExitListHeader;
use crate::header::scene::variant::lighting::LightingHeader;
use crate::header::scene::variant::pathway::PathwaysHeader;
use crate::header::scene::variant::room::RoomListHeader;
use crate::header::scene::variant::skybox::SceneSkyboxHeader;
use crate::header::scene::variant::sound::SceneSoundHeader;
use crate::header::scene::variant::special::SpecialObjectsHeader;
use crate::header::scene::variant::start::StartPositionsHeader;
use crate::header::scene::variant::transition::TransitionActorsHeader;
use crate::header::scene::SceneHeader;
use crate::reflect::{Field, Reflect};

pub mod camera;
pub mod collision;
pub mod entrance;
pub mod exit;
pub mod lighting;
pub mod pathway;
pub mod room;
pub mod skybox;
pub mod sound;
pub mod special;
pub mod start;
pub mod transition;

#[derive(Clone, Copy, Debug)]
pub enum SceneHeaderVariant<'a> {
    StartPositions(StartPositionsHeader<'a>),
    Collision(CollisionHeader<'a>),
    RoomList(RoomListHeader<'a>),
    EntranceList(EntranceListHeader<'a>),
    SpecialObjects(SpecialObjectsHeader<'a>),
    Pathways(PathwaysHeader<'a>),
    TransitionActors(TransitionActorsHeader<'a>),
    Lighting(LightingHeader<'a>),
    Skybox(SceneSkyboxHeader<'a>),
    ExitList(ExitListHeader<'a>),
    End,
    Sound(SceneSoundHeader<'a>),
    AlternateHeaders(AlternateHeadersHeader<'a>),
    CameraAndWorldMap(CameraAndWorldMapHeader<'a>),
}

impl<'a> Reflect for SceneHeaderVariant<'a> {
    fn name(&self) -> Cow<'static, str> {
        // TODO: dispatch to variant
        Cow::Borrowed("SceneHeader")
    }

    fn size(&self) -> u32 {
        SceneHeader::SIZE as u32
    }

    fn addr(&self) -> VromAddr {
        // TODO: this is fake
        VromAddr(0x123456)
    }

    fn iter_fields(&self) -> Box<dyn Iterator<Item = Box<dyn Field + '_>> + '_> {
        Box::new(std::iter::empty())
    }
}
