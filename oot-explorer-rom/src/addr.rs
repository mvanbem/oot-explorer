use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// An address in ROM.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RomAddr(pub u32);

impl Debug for RomAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RomAddr(0x{:08x})", self.0)
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
