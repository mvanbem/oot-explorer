use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::slice::StructReader;

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
