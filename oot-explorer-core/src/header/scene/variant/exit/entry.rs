use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

use crate::slice::StructReader;

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
