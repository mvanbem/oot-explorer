use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A borrowed reference to an entire Ocarina of Time ROM.
#[derive(Clone, Copy)]
pub struct Rom<'a> {
    data: &'a [u8],
}

impl<'a> Rom<'a> {
    /// Wraps a byte slice.
    pub fn new(data: &'a [u8]) -> Rom<'a> {
        assert!(data.len() <= std::u32::MAX as usize);
        Rom { data }
    }

    /// Returns the length of the ROM as a RomAddr; the lowest invalid address.
    pub fn len(self) -> RomAddr {
        RomAddr(self.data.len() as u32)
    }

    pub fn slice<'b>(self, addr: RomAddr, size: u32) -> &'b [u8]
    where
        'a: 'b,
    {
        &self.data[addr.0 as usize..(addr.0 + size) as usize]
    }

    pub fn read_u32_at(self, addr: RomAddr) -> u32 {
        let mut r = &self.data[addr.0 as usize..];
        r.read_u32::<BigEndian>().unwrap()
    }
}

impl<'a> Debug for Rom<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ROM(len 0x{:08x})", self.data.len())
    }
}

/// An address in ROM.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct RomAddr(pub u32);

impl Debug for RomAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ROM(0x{:08x})", self.0)
    }
}

impl Add<u32> for RomAddr {
    type Output = RomAddr;
    fn add(self, rhs: u32) -> RomAddr {
        RomAddr(self.0 + rhs)
    }
}

impl AddAssign<u32> for RomAddr {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sub<RomAddr> for RomAddr {
    type Output = u32;
    fn sub(self, rhs: RomAddr) -> u32 {
        self.0 - rhs.0
    }
}

impl Sub<u32> for RomAddr {
    type Output = RomAddr;
    fn sub(self, rhs: u32) -> RomAddr {
        RomAddr(self.0 - rhs)
    }
}

impl SubAssign<u32> for RomAddr {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}
