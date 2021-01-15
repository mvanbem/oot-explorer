use std::fmt::{self, Debug, Formatter};

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
    pub fn new(data: &'a [u8]) -> RoomSoundHeader<'a> {
        RoomSoundHeader { data }
    }

    pub fn echo(self) -> u8 {
        self.data[7]
    }
}
