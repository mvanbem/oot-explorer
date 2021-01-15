use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::special::elf_message::ElfMessage;
use crate::header::scene::variant::special::global_object::GlobalObject;

pub mod elf_message;
pub mod global_object;

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
    pub fn new(data: &'a [u8]) -> SpecialObjectsHeader<'a> {
        SpecialObjectsHeader { data }
    }

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
