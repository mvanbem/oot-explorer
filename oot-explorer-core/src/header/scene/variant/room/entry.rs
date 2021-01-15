use byteorder::{BigEndian, ReadBytesExt};
use scoped_owner::ScopedOwner;
use std::fmt::{self, Debug, Formatter};

use crate::fs::{LazyFileSystem, VromAddr};
use crate::room::Room;
use crate::slice::StructReader;

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
