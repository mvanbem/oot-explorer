use byteorder::{BigEndian, ReadBytesExt};
use scoped_owner::ScopedOwner;
use std::fmt::{self, Debug, Formatter};

use crate::fs::{LazyFileSystem, VromAddr};
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::struct_::{FieldDescriptor, StructDescriptor, StructFieldLocation};
use crate::reflect::type_::TypeDescriptor;
use crate::room::Room;

pub const ROOM_LIST_ENTRY_DESC: TypeDescriptor = TypeDescriptor::Struct(&StructDescriptor {
    name: "RoomListEntry",
    size: Some(8),
    is_end: None,
    fields: &[
        FieldDescriptor {
            name: "start",
            location: StructFieldLocation::Simple { offset: 0 },
            desc: TypeDescriptor::Primitive(PrimitiveType::U32),
        },
        FieldDescriptor {
            name: "end",
            location: StructFieldLocation::Simple { offset: 4 },
            desc: TypeDescriptor::Primitive(PrimitiveType::U32),
        },
    ],
});

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

impl<'a> Instantiate<'a> for RoomListEntry<'a> {
    fn new(data: &'a [u8]) -> RoomListEntry<'a> {
        RoomListEntry { data }
    }
}

impl<'a> ReflectSized for RoomListEntry<'a> {
    const SIZE: usize = 8;
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
