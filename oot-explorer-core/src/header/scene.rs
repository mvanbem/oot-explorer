use std::borrow::Cow;

use crate::fs::VromAddr;
use crate::header::alternate::AlternateHeadersHeader;
use crate::header::scene::type_::SceneHeaderType;
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
use crate::header::scene::variant::SceneHeaderVariant;
use crate::reflect::{Field, Reflect, Sourced, Value};

pub mod iter;
pub mod type_;
pub mod variant;

#[derive(Clone, Copy)]
pub struct SceneHeader<'a> {
    data: &'a [u8],
}

impl<'a> SceneHeader<'a> {
    pub const SIZE: usize = 8;

    pub fn new(data: &'a [u8]) -> SceneHeader<'a> {
        SceneHeader { data }
    }

    pub fn type_(self) -> SceneHeaderType {
        SceneHeaderType(self.data[0])
    }

    pub fn scene_header(self) -> SceneHeaderVariant<'a> {
        let data = self.data;
        match self.type_() {
            SceneHeaderType::START_POSITIONS => {
                SceneHeaderVariant::StartPositions(StartPositionsHeader::new(data))
            }
            SceneHeaderType::COLLISION => SceneHeaderVariant::Collision(CollisionHeader::new(data)),
            SceneHeaderType::ROOM_LIST => SceneHeaderVariant::RoomList(RoomListHeader::new(data)),
            SceneHeaderType::ENTRANCE_LIST => {
                SceneHeaderVariant::EntranceList(EntranceListHeader::new(data))
            }
            SceneHeaderType::SPECIAL_OBJECTS => {
                SceneHeaderVariant::SpecialObjects(SpecialObjectsHeader::new(data))
            }
            SceneHeaderType::PATHWAYS => SceneHeaderVariant::Pathways(PathwaysHeader::new(data)),
            SceneHeaderType::TRANSITION_ACTORS => {
                SceneHeaderVariant::TransitionActors(TransitionActorsHeader::new(data))
            }
            SceneHeaderType::LIGHTING => SceneHeaderVariant::Lighting(LightingHeader::new(data)),
            SceneHeaderType::SKYBOX => SceneHeaderVariant::Skybox(SceneSkyboxHeader::new(data)),
            SceneHeaderType::EXIT_LIST => SceneHeaderVariant::ExitList(ExitListHeader::new(data)),
            SceneHeaderType::END => SceneHeaderVariant::End,
            SceneHeaderType::SOUND => SceneHeaderVariant::Sound(SceneSoundHeader::new(data)),
            SceneHeaderType::ALTERNATE_HEADERS => {
                SceneHeaderVariant::AlternateHeaders(AlternateHeadersHeader::new(data))
            }
            SceneHeaderType::CAMERA_AND_WORLD_MAP => {
                SceneHeaderVariant::CameraAndWorldMap(CameraAndWorldMapHeader::new(data))
            }
            type_ => panic!("unexpected scene header type: 0x{:02x}", type_.0),
        }
    }
}

impl<'a> Reflect for Sourced<SceneHeader<'a>> {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Header")
    }

    fn size(&self) -> u32 {
        SceneHeader::SIZE as u32
    }

    fn addr(&self) -> VromAddr {
        Sourced::addr(self)
    }

    fn iter_fields(&self) -> Box<dyn Iterator<Item = Box<dyn Field + '_>> + '_> {
        Box::new(HeaderFieldsIter {
            header: self.clone(),
            index: 0,
        })
    }
}

#[derive(Clone)]
struct HeaderFieldsIter<'a> {
    header: Sourced<SceneHeader<'a>>,
    index: usize,
}

impl<'a> Iterator for HeaderFieldsIter<'a> {
    type Item = Box<dyn Field + 'a>;

    fn next(&mut self) -> Option<Box<dyn Field + 'a>> {
        if self.index < 1 {
            let field = self.clone();
            self.index += 1;
            Some(Box::new(field))
        } else {
            None
        }
    }
}

impl<'a> Field for HeaderFieldsIter<'a> {
    fn size(&self) -> u32 {
        match self.index {
            0 => 1,
            _ => unreachable!(),
        }
    }

    fn addr(&self) -> VromAddr {
        match self.index {
            0 => self.header.addr(),
            _ => unreachable!(),
        }
    }

    fn name(&self) -> Cow<'static, str> {
        match self.index {
            0 => Cow::Borrowed("code"),
            _ => unreachable!(),
        }
    }

    fn try_get(&self) -> Option<Value> {
        match self.index {
            // TODO: Enum types!
            0 => Some(Value::U8(self.header.type_().0)),
            _ => unreachable!(),
        }
    }
}
