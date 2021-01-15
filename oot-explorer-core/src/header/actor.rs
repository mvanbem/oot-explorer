use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::slice::StructReader;

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

    pub fn angle_x(self) -> i16 {
        (&self.data[8..]).read_i16::<BigEndian>().unwrap()
    }

    pub fn angle_y(self) -> i16 {
        (&self.data[10..]).read_i16::<BigEndian>().unwrap()
    }

    pub fn angle_z(self) -> i16 {
        (&self.data[12..]).read_i16::<BigEndian>().unwrap()
    }

    pub fn init(self) -> u16 {
        (&self.data[14..]).read_u16::<BigEndian>().unwrap()
    }
}
