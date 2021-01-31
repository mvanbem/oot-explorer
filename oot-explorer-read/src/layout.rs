use oot_explorer_rom::RomAddr;
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::ReadError;

/// Types that read a value of statically known size and alignment.
pub trait Layout {
    const SIZE: u32;
    const ALIGN_BITS: u32 = Self::SIZE.trailing_zeros();
}

impl Layout for bool {
    const SIZE: u32 = 1;
}

impl Layout for u8 {
    const SIZE: u32 = 1;
}

impl Layout for i8 {
    const SIZE: u32 = 1;
}

impl Layout for u16 {
    const SIZE: u32 = 2;
}

impl Layout for i16 {
    const SIZE: u32 = 2;
}

impl Layout for u32 {
    const SIZE: u32 = 4;
}

impl Layout for i32 {
    const SIZE: u32 = 4;
}

impl Layout for RomAddr {
    const SIZE: u32 = 4;
}

impl Layout for VromAddr {
    const SIZE: u32 = 4;
}

pub fn check_alignment<T: Layout>(addr: VromAddr) -> Result<(), ReadError> {
    if addr.0.trailing_zeros() >= T::ALIGN_BITS {
        Ok(())
    } else {
        Err(ReadError::Misaligned {
            align_bits: T::ALIGN_BITS,
            addr,
        })
    }
}

pub fn aligned_data<T: Layout>(vrom: Vrom<'_>, addr: VromAddr) -> Result<&[u8], ReadError> {
    check_alignment::<T>(addr)?;
    Ok(vrom.slice(addr..addr + T::SIZE)?)
}
