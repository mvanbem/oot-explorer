use scoped_owner::ScopedOwner;
use thiserror::Error;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::header::alternate::AlternateHeadersHeader;
use crate::header::scene::type_::{SceneHeaderType, SCENE_HEADER_TYPE_DESC};
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
use crate::header::scene::variant::start::{StartPositionsHeader, START_POSITIONS_DESC};
use crate::header::scene::variant::transition::TransitionActorsHeader;
use crate::header::scene::variant::SceneHeaderVariant;
use crate::reflect::struct_::UnionDescriptor;
use crate::reflect::type_::TypeDescriptor;

pub mod iter;
pub mod type_;
pub mod variant;

const SCENE_HEADER_INNER_DESC: &UnionDescriptor = &UnionDescriptor {
    name: "SceneHeader",
    size: Some(SceneHeader::SIZE as u32),
    is_end: Some(scene_header_is_end),
    discriminant_offset: 0,
    discriminant_desc: SCENE_HEADER_TYPE_DESC,
    variants: &[/* 0x00 */ Some(START_POSITIONS_DESC)],
};

pub const SCENE_HEADER_DESC: TypeDescriptor = TypeDescriptor::Union(SCENE_HEADER_INNER_DESC);

fn scene_header_is_end<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    addr: VromAddr,
) -> bool {
    let data = match fs.get_virtual_slice(scope, addr..addr + SceneHeader::SIZE as u32) {
        Ok(data) => data,
        Err(VirtualSliceError::OutOfRange { .. }) => return true,
    };
    SceneHeader::new(data).type_() == SceneHeaderType::END
}

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

    pub fn variant(self) -> Result<SceneHeaderVariant<'a>, SceneHeaderVariantError> {
        let data = self.data;
        Ok(match self.type_() {
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
            type_ => return Err(SceneHeaderVariantError::Unknown(type_)),
        })
    }
}

#[derive(Debug, Error)]
pub enum SceneHeaderVariantError {
    #[error("unknown scene header type: 0x{:02x}", (.0).0)]
    Unknown(SceneHeaderType),
}
