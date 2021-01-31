use byteorder::{BigEndian, ReadBytesExt};
use oot_explorer_rom::RomAddr;
use oot_explorer_segment::SegmentAddr;
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::ReadError;

/// Types that can be constructed with VROM data and an address.
pub trait FromVrom: Sized {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError>;
}

impl FromVrom for bool {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(u8::from_vrom(vrom, addr)? != 0)
    }
}

impl FromVrom for u8 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 1)?.read_u8().unwrap())
    }
}

impl FromVrom for i8 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 1)?.read_i8().unwrap())
    }
}

impl FromVrom for u16 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 2)?.read_u16::<BigEndian>().unwrap())
    }
}

impl FromVrom for i16 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 2)?.read_i16::<BigEndian>().unwrap())
    }
}

impl FromVrom for u32 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 4)?.read_u32::<BigEndian>().unwrap())
    }
}

impl FromVrom for i32 {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(vrom.slice(addr..addr + 4)?.read_i32::<BigEndian>().unwrap())
    }
}

impl FromVrom for RomAddr {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(RomAddr(u32::from_vrom(vrom, addr)?))
    }
}

impl FromVrom for VromAddr {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(VromAddr(u32::from_vrom(vrom, addr)?))
    }
}

impl FromVrom for SegmentAddr {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(SegmentAddr(u32::from_vrom(vrom, addr)?))
    }
}
