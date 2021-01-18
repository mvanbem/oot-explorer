use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::reflect::primitive::PrimitiveType;
use crate::reflect::struct_::{FieldDescriptor, StructDescriptor, StructFieldLocation};
use crate::reflect::type_::TypeDescriptor;
use crate::slice::StructReader;

pub const ACTOR_DESC: TypeDescriptor = TypeDescriptor::Struct(&StructDescriptor {
    name: "Actor",
    size: Some(0x10),
    is_end: None,
    fields: &[
        FieldDescriptor {
            name: "actor_number",
            location: StructFieldLocation::Simple { offset: 0 },
            desc: TypeDescriptor::Primitive(PrimitiveType::U16),
        },
        FieldDescriptor {
            name: "pos_x",
            location: StructFieldLocation::Simple { offset: 2 },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "pos_y",
            location: StructFieldLocation::Simple { offset: 4 },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "pos_z",
            location: StructFieldLocation::Simple { offset: 6 },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "angle_x",
            location: StructFieldLocation::Simple { offset: 8 },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "angle_y",
            location: StructFieldLocation::Simple { offset: 0xa },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "angle_z",
            location: StructFieldLocation::Simple { offset: 0xc },
            desc: TypeDescriptor::Primitive(PrimitiveType::I16),
        },
        FieldDescriptor {
            name: "init",
            location: StructFieldLocation::Simple { offset: 0xe },
            desc: TypeDescriptor::Primitive(PrimitiveType::U16),
        },
    ],
});

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
