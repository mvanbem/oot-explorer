use ::byteorder::{BigEndian, ReadBytesExt};

pub trait Instantiate<'scope> {
    fn new(data: &'scope [u8]) -> Self;
}

impl<'scope> Instantiate<'scope> for bool {
    fn new(data: &'scope [u8]) -> Self {
        data[0] != 0
    }
}

impl<'scope> Instantiate<'scope> for u8 {
    fn new(data: &'scope [u8]) -> Self {
        data[0]
    }
}

impl<'scope> Instantiate<'scope> for i8 {
    fn new(data: &'scope [u8]) -> Self {
        data[0] as i8
    }
}

impl<'scope> Instantiate<'scope> for u16 {
    fn new(data: &'scope [u8]) -> Self {
        ReadBytesExt::read_u16::<BigEndian>(&mut &data[..2]).unwrap()
    }
}

impl<'scope> Instantiate<'scope> for i16 {
    fn new(data: &'scope [u8]) -> Self {
        ReadBytesExt::read_i16::<BigEndian>(&mut &data[..2]).unwrap()
    }
}

impl<'scope> Instantiate<'scope> for u32 {
    fn new(data: &'scope [u8]) -> Self {
        ReadBytesExt::read_u32::<BigEndian>(&mut &data[..4]).unwrap()
    }
}

impl<'scope> Instantiate<'scope> for i32 {
    fn new(data: &'scope [u8]) -> Self {
        ReadBytesExt::read_i32::<BigEndian>(&mut &data[..4]).unwrap()
    }
}
