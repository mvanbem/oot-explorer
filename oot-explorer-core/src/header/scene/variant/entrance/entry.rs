use std::fmt::{self, Debug, Formatter};

use crate::slice::StructReader;

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
