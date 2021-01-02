use crate::collision::Collision;
use crate::fs::{LazyFileSystem, VromAddr};
use crate::mesh::Mesh;
use crate::object::ObjectId;
use crate::room::Room;
use crate::scene::Lighting;
use crate::segment::{SegmentAddr, SegmentCtx};
use crate::slice::{Slice, StructReader};
use byteorder::{BigEndian, ReadBytesExt};
use scoped_owner::ScopedOwner;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct Header<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Header<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Header")
            .field("code", &self.code())
            .finish()
    }
}
impl<'a> Header<'a> {
    pub const SIZE: usize = 8;

    pub fn new(data: &'a [u8]) -> Header<'a> {
        Header { data }
    }
    pub fn code(self) -> u8 {
        self.data[0]
    }

    pub fn is_end(self) -> bool {
        self.code() == 0x14
    }
    pub fn scene_header(self) -> SceneHeader<'a> {
        let data = self.data;
        match self.code() {
            0x00 => SceneHeader::StartPositions(StartPositionsHeader { data }),
            0x03 => SceneHeader::Collision(CollisionHeader { data }),
            0x04 => SceneHeader::RoomList(RoomListHeader { data }),
            0x06 => SceneHeader::EntranceList(EntranceListHeader { data }),
            0x07 => SceneHeader::SpecialObjects(SpecialObjectsHeader { data }),
            0x0d => SceneHeader::Pathways(PathwaysHeader { data }),
            0x0e => SceneHeader::TransitionActors(TransitionActorsHeader { data }),
            0x0f => SceneHeader::Lighting(LightingHeader { data }),
            0x11 => SceneHeader::Skybox(SceneSkyboxHeader { data }),
            0x13 => SceneHeader::ExitList(ExitListHeader { data }),
            0x14 => SceneHeader::End,
            0x15 => SceneHeader::Sound(SceneSoundHeader { data }),
            0x18 => SceneHeader::AlternateHeaders(AlternateHeadersHeader { data }),
            0x19 => SceneHeader::CameraAndWorldMap(CameraAndWorldMapHeader { data }),
            code => panic!("unexpected scene header code: 0x{:02x}", code),
        }
    }
    pub fn room_header(self) -> RoomHeader<'a> {
        let data = self.data;
        match self.code() {
            0x01 => RoomHeader::ActorList(ActorListHeader { data }),
            0x05 => RoomHeader::Wind(WindHeader { data }),
            0x08 => RoomHeader::Behavior(RoomBehaviorHeader { data }),
            0x0a => RoomHeader::Mesh(MeshHeader { data }),
            0x0b => RoomHeader::ObjectList(ObjectListHeader { data }),
            0x10 => RoomHeader::Time(TimeHeader { data }),
            0x12 => RoomHeader::Skybox(RoomSkyboxHeader { data }),
            0x14 => RoomHeader::End,
            0x16 => RoomHeader::Sound(RoomSoundHeader { data }),
            0x18 => RoomHeader::AlternateHeaders(AlternateHeadersHeader { data }),
            code => panic!("unexpected room header code: 0x{:02x}", code),
        }
    }
}

pub struct Iter<'a> {
    data: &'a [u8],
}
impl<'a> Iter<'a> {
    pub fn new(data: &'a [u8]) -> Iter<'a> {
        Iter { data }
    }
}
impl<'a> Iterator for Iter<'a> {
    type Item = Header<'a>;
    fn next(&mut self) -> Option<Header<'a>> {
        let header = Header::new(self.data);
        if header.is_end() {
            None
        } else {
            self.data = &self.data[Header::SIZE..];
            Some(header)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SceneHeader<'a> {
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

#[derive(Clone, Copy, Debug)]
pub enum RoomHeader<'a> {
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

// Header 0x00
#[derive(Clone, Copy)]
pub struct StartPositionsHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for StartPositionsHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("StartPositionsHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> StartPositionsHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn actor_list(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, Actor<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x01
#[derive(Clone, Copy)]
pub struct ActorListHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for ActorListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ActorListHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> ActorListHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn actor_list(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, Actor<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x03
#[derive(Clone, Copy)]
pub struct CollisionHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for CollisionHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CollisionHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> CollisionHeader<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn collision(self, segment_ctx: &'a SegmentCtx) -> Collision<'a> {
        Collision::new(segment_ctx.resolve(self.ptr()).unwrap())
    }
}

// Header 0x04
#[derive(Clone, Copy)]
pub struct RoomListHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for RoomListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomListHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> RoomListHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn room_list(self, segment_ctx: &SegmentCtx<'a>) -> Slice<'a, RoomListEntry<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x05
#[derive(Clone, Copy)]
pub struct WindHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for WindHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("WindHeader")
            .field("west", &self.west())
            .field("up", &self.up())
            .field("south", &self.south())
            .field("strength", &self.strength())
            .finish()
    }
}
impl<'a> WindHeader<'a> {
    pub fn west(self) -> i8 {
        self.data[4] as i8
    }
    pub fn up(self) -> i8 {
        self.data[5] as i8
    }
    pub fn south(self) -> i8 {
        self.data[6] as i8
    }
    pub fn strength(self) -> u8 {
        self.data[7]
    }
}

// Header 0x06
#[derive(Clone, Copy)]
pub struct EntranceListHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for EntranceListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("EntranceListHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> EntranceListHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn entrance_list_with_len(
        self,
        segment_ctx: &'a SegmentCtx,
        len: usize,
    ) -> Slice<'a, Entrance<'a>> {
        Slice::new(segment_ctx.resolve(self.ptr()).unwrap(), len)
    }
}

// Header 0x07
#[derive(Clone, Copy)]
pub struct SpecialObjectsHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for SpecialObjectsHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SpecialObjectsHeader")
            .field("elf_message", &self.elf_message())
            .field("global_object", &self.global_object())
            .finish()
    }
}
impl<'a> SpecialObjectsHeader<'a> {
    pub fn raw_elf_message(self) -> u8 {
        self.data[1]
    }
    pub fn raw_global_object(self) -> u16 {
        (&self.data[6..]).read_u16::<BigEndian>().unwrap()
    }

    pub fn elf_message(self) -> ElfMessage {
        ElfMessage::parse(self.raw_elf_message())
    }
    pub fn global_object(self) -> GlobalObject {
        GlobalObject::parse(self.raw_global_object())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfMessage {
    None,
    Field,
    Ydan,
}
impl ElfMessage {
    fn parse(value: u8) -> ElfMessage {
        match value {
            0x00 => ElfMessage::None,
            0x01 => ElfMessage::Field,
            0x02 => ElfMessage::Ydan,
            _ => panic!("unexpected elf message value: 0x{:02x}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalObject {
    None,
    Field,
    Dangeon,
}
impl GlobalObject {
    fn parse(value: u16) -> GlobalObject {
        match value {
            0x0000 => GlobalObject::None,
            0x0002 => GlobalObject::Field,
            0x0003 => GlobalObject::Dangeon,
            _ => panic!("unexpected global object value: 0x{:04x}", value),
        }
    }
}

// Header 0x08
#[derive(Clone, Copy)]
pub struct RoomBehaviorHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for RoomBehaviorHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomBehaviorHeader")
            .field("x", &self.x())
            .field("disable_warp_songs", &self.disable_warp_songs())
            .field("show_invisible_actors", &self.show_invisible_actors())
            .field("idle_animation_or_heat", &self.idle_animation_or_heat())
            .finish()
    }
}
impl<'a> RoomBehaviorHeader<'a> {
    /// Affects Sun's Song, backflipping with A.
    pub fn x(self) -> u8 {
        self.data[1]
    }
    pub fn disable_warp_songs(self) -> u8 {
        self.data[6] >> 4
    }
    pub fn show_invisible_actors(self) -> bool {
        self.data[6] & 1 == 1
    }
    pub fn idle_animation_or_heat(self) -> u8 {
        self.data[7]
    }
}

// Header 0x0a
#[derive(Clone, Copy)]
pub struct MeshHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for MeshHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("MeshHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> MeshHeader<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn mesh(self, segment_ctx: &'a SegmentCtx) -> Mesh<'a> {
        Mesh::new(segment_ctx.resolve(self.ptr()).unwrap())
    }
}

// Header 0x0b
#[derive(Clone, Copy)]
pub struct ObjectListHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for ObjectListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ObjectListHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> ObjectListHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn objects(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, ObjectId> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x0d
#[derive(Clone, Copy)]
pub struct PathwaysHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for PathwaysHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("PathwaysHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> PathwaysHeader<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    // TODO: Expose and parse these! Count is not explicitly stored.
}

// Header 0x0e
#[derive(Clone, Copy)]
pub struct TransitionActorsHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for TransitionActorsHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TransitionActorsHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> TransitionActorsHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn transition_actors(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, TransitionActor<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x0f
#[derive(Clone, Copy)]
pub struct LightingHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for LightingHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("LightingHeader")
            .field("count", &self.count())
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> LightingHeader<'a> {
    pub fn count(self) -> u8 {
        self.data[1]
    }
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn lighting(self, segment_ctx: &'a SegmentCtx) -> Slice<'a, Lighting<'a>> {
        Slice::new(
            segment_ctx.resolve(self.ptr()).unwrap(),
            self.count() as usize,
        )
    }
}

// Header 0x10
#[derive(Clone, Copy)]
pub struct TimeHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for TimeHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TimeHeader")
            .field("time_override", &self.time_override())
            .field("time_speed", &self.time_speed())
            .finish()
    }
}
impl<'a> TimeHeader<'a> {
    pub fn time_override(self) -> Option<u16> {
        let time = (&self.data[4..]).read_u16::<BigEndian>().unwrap();
        if time == 0xffff {
            None
        } else {
            Some(time)
        }
    }
    pub fn time_speed(self) -> i8 {
        self.data[6] as i8
    }
}

// Header 0x11
#[derive(Clone, Copy)]
pub struct SceneSkyboxHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for SceneSkyboxHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SceneSkyboxHeader")
            .field("skybox", &self.skybox())
            .field("cloudy", &self.cloudy())
            .field("indoor_lighting", &self.indoor_lighting())
            .finish()
    }
}
impl<'a> SceneSkyboxHeader<'a> {
    pub fn skybox(self) -> u8 {
        self.data[4]
    }
    pub fn cloudy(self) -> bool {
        self.data[5] != 0
    }
    pub fn indoor_lighting(self) -> bool {
        self.data[6] != 0
    }
}

// Header 0x12
#[derive(Clone, Copy)]
pub struct RoomSkyboxHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for RoomSkyboxHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomSkyboxHeader")
            .field("disable_sky", &self.disable_sky())
            .field("disable_sun_moon", &self.disable_sun_moon())
            .finish()
    }
}
impl<'a> RoomSkyboxHeader<'a> {
    pub fn disable_sky(self) -> bool {
        self.data[4] != 0
    }
    pub fn disable_sun_moon(self) -> bool {
        self.data[5] != 0
    }
}

// Header 0x13
#[derive(Clone, Copy)]
pub struct ExitListHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for ExitListHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ExitListHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> ExitListHeader<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn exit_list_with_len(
        self,
        segment_ctx: &'a SegmentCtx,
        len: usize,
    ) -> Slice<'a, Exit<'a>> {
        Slice::new(segment_ctx.resolve(self.ptr()).unwrap(), len)
    }
}

#[derive(Clone, Copy)]
pub struct Exit<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Exit<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Exit(0x{:04x})", self.get())
    }
}
impl<'a> StructReader<'a> for Exit<'a> {
    const SIZE: usize = 2;
    fn new(data: &'a [u8]) -> Exit<'a> {
        Exit { data }
    }
}
impl<'a> Exit<'a> {
    fn get(self) -> u16 {
        (&self.data[..]).read_u16::<BigEndian>().unwrap()
    }
}

// Header 0x15
#[derive(Clone, Copy)]
pub struct SceneSoundHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for SceneSoundHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SceneSoundHeader")
            .field("settings", &self.settings())
            .field("night_sound", &self.night_sound())
            .field("music", &self.music())
            .finish()
    }
}
impl<'a> SceneSoundHeader<'a> {
    pub fn settings(self) -> u8 {
        self.data[1]
    }
    pub fn night_sound(self) -> u8 {
        self.data[6]
    }
    pub fn music(self) -> u8 {
        self.data[7]
    }
}

// Header 0x16
#[derive(Clone, Copy)]
pub struct RoomSoundHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for RoomSoundHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomSoundHeader")
            .field("echo", &self.echo())
            .finish()
    }
}
impl<'a> RoomSoundHeader<'a> {
    pub fn echo(self) -> u8 {
        self.data[7]
    }
}

// Header 0x18
#[derive(Clone, Copy)]
pub struct AlternateHeadersHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for AlternateHeadersHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("AlternateHeadersHeader")
            .field("ptr", &self.ptr())
            .finish()
    }
}
impl<'a> AlternateHeadersHeader<'a> {
    pub fn ptr(self) -> SegmentAddr {
        SegmentAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    // TODO: Expose and parse these! Count is not explicitly stored.
}

// Header 0x19
#[derive(Clone, Copy)]
pub struct CameraAndWorldMapHeader<'a> {
    data: &'a [u8],
}
impl<'a> Debug for CameraAndWorldMapHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CameraAndWorldMapHeader")
            .field("camera", &self.camera())
            .field("world_map_location", &self.world_map_location())
            .finish()
    }
}
impl<'a> CameraAndWorldMapHeader<'a> {
    // Raw getters
    pub fn raw_camera(self) -> u8 {
        self.data[1]
    }
    pub fn world_map_location(self) -> u8 {
        self.data[7]
    }

    // Interpreted getters
    pub fn camera(self) -> Camera {
        Camera::parse(self.raw_camera())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Camera {
    Free,
    FixedWithAlternate,
    RotateWithAlternate,
    Fixed,
    Rotate,
    ShootingGallery,
}
impl Camera {
    pub fn parse(value: u8) -> Camera {
        match value {
            0x00 => Camera::Free,
            0x10 => Camera::FixedWithAlternate,
            0x20 => Camera::RotateWithAlternate,
            0x30 => Camera::Fixed,
            0x40 => Camera::Rotate,
            0x50 => Camera::ShootingGallery,
            _ => panic!("unexpected camera value: 0x{:02x}", value),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Actor<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Actor<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Actor")
            .field("actor_number", &self.actor_number())
            .field("pos_x", &self.pos_x())
            .field("pos_y", &self.pos_y())
            .field("pos_z", &self.pos_z())
            .finish()
    }
}
impl<'a> StructReader<'a> for Actor<'a> {
    const SIZE: usize = 16;
    fn new(data: &'a [u8]) -> Actor<'a> {
        Actor { data }
    }
}
impl<'a> Actor<'a> {
    pub fn actor_number(self) -> u16 {
        (&self.data[..]).read_u16::<BigEndian>().unwrap()
    }
    pub fn pos_x(self) -> i16 {
        (&self.data[2..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn pos_y(self) -> i16 {
        (&self.data[4..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn pos_z(self) -> i16 {
        (&self.data[6..]).read_i16::<BigEndian>().unwrap()
    }
    // TODO: other properties
}

#[derive(Clone, Copy)]
pub struct Entrance<'a> {
    data: &'a [u8],
}
impl<'a> Debug for Entrance<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Entrance")
            .field("start_position", &self.start_position())
            .field("room", &self.room())
            .finish()
    }
}
impl<'a> StructReader<'a> for Entrance<'a> {
    const SIZE: usize = 2;
    fn new(data: &'a [u8]) -> Entrance<'a> {
        Entrance { data }
    }
}
impl<'a> Entrance<'a> {
    pub fn start_position(self) -> u8 {
        self.data[0]
    }
    pub fn room(self) -> u8 {
        self.data[1]
    }
}

#[derive(Clone, Copy)]
pub struct TransitionActor<'a> {
    data: &'a [u8],
}
impl<'a> Debug for TransitionActor<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TransitionActor")
            .field("room_from_front", &self.room_from_front())
            .field("camera_from_front", &self.camera_from_front())
            .field("room_from_back", &self.room_from_back())
            .field("camera_from_back", &self.camera_from_back())
            .field("actor_number", &self.actor_number())
            .field("pos_x", &self.pos_x())
            .field("pos_y", &self.pos_y())
            .field("pos_z", &self.pos_z())
            .field("rot_y", &self.rot_y())
            .field("init", &self.init())
            .finish()
    }
}
impl<'a> StructReader<'a> for TransitionActor<'a> {
    const SIZE: usize = 16;
    fn new(data: &'a [u8]) -> TransitionActor<'a> {
        TransitionActor { data }
    }
}
impl<'a> TransitionActor<'a> {
    pub fn room_from_front(self) -> u8 {
        self.data[0]
    }
    pub fn camera_from_front(self) -> u8 {
        self.data[1]
    }
    pub fn room_from_back(self) -> u8 {
        self.data[2]
    }
    pub fn camera_from_back(self) -> u8 {
        self.data[3]
    }
    pub fn actor_number(self) -> i16 {
        (&self.data[4..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn pos_x(self) -> i16 {
        (&self.data[6..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn pos_y(self) -> i16 {
        (&self.data[8..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn pos_z(self) -> i16 {
        (&self.data[10..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn rot_y(self) -> i16 {
        (&self.data[12..]).read_i16::<BigEndian>().unwrap()
    }
    pub fn init(self) -> i16 {
        (&self.data[14..]).read_i16::<BigEndian>().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct RoomListEntry<'a> {
    data: &'a [u8],
}
impl<'a> Debug for RoomListEntry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomListEntry")
            .field("start", &self.start())
            .field("end", &self.end())
            .finish()
    }
}
impl<'a> StructReader<'a> for RoomListEntry<'a> {
    const SIZE: usize = 8;

    fn new(data: &'a [u8]) -> RoomListEntry<'a> {
        RoomListEntry { data }
    }
}
impl<'a> RoomListEntry<'a> {
    pub fn start(self) -> VromAddr {
        VromAddr((&self.data[..]).read_u32::<BigEndian>().unwrap())
    }
    pub fn end(self) -> VromAddr {
        VromAddr((&self.data[4..]).read_u32::<BigEndian>().unwrap())
    }

    pub fn room(self, scope: &'a ScopedOwner, fs: &mut LazyFileSystem<'a>) -> Room<'a> {
        Room::new(
            self.start(),
            fs.get_virtual_slice_or_die(scope, self.start()..self.end()),
        )
    }
}
