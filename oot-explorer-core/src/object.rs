use crate::slice::StructReader;
use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(pub u16);
impl Debug for ObjectId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ObjectId(0x{:04x})", self.0)
    }
}
impl<'a> StructReader<'a> for ObjectId {
    const SIZE: usize = 2;
    fn new(mut data: &'a [u8]) -> ObjectId {
        ObjectId(data.read_u16::<BigEndian>().unwrap())
    }
}
