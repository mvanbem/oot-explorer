use std::fmt::{self, Debug};
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::VromError;

/// An address in VROM.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VromAddr(pub u32);

impl VromAddr {
    pub fn checked_add(self, offset: u32) -> Result<VromAddr, VromError> {
        match self.0.checked_add(offset) {
            Some(result) => Ok(VromAddr(result)),
            None => Err(VromError::VromAddrOverflow { addr: self, offset }),
        }
    }
}

impl Debug for VromAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VromAddr(0x{:08x})", self.0)
    }
}

impl Add<u32> for VromAddr {
    type Output = VromAddr;

    fn add(self, rhs: u32) -> VromAddr {
        VromAddr(self.0 + rhs)
    }
}

impl AddAssign<u32> for VromAddr {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sub<VromAddr> for VromAddr {
    type Output = u32;

    fn sub(self, rhs: VromAddr) -> u32 {
        self.0 - rhs.0
    }
}

impl Sub<u32> for VromAddr {
    type Output = VromAddr;

    fn sub(self, rhs: u32) -> VromAddr {
        VromAddr(self.0 - rhs)
    }
}

impl SubAssign<u32> for VromAddr {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}
