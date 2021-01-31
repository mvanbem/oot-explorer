use oot_explorer_read::{Layout, ReadError, VromProxy};
use oot_explorer_vrom::{Vrom, VromAddr};
use std::ops::{Deref, Range};

#[derive(Clone, Copy)]
pub struct Sourced<T> {
    addr: VromAddr,
    value: T,
}

impl<T> Sourced<T> {
    pub fn new(addr: VromAddr, value: T) -> Sourced<T> {
        Sourced { addr, value }
    }

    pub fn addr(&self) -> VromAddr {
        self.addr
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Sourced<T>
where
    T: Layout,
{
    pub fn vrom_range(&self) -> Range<VromAddr> {
        self.addr..self.addr + T::SIZE as u32
    }
}

impl<T> Deref for Sourced<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Copy)]
pub struct RangeSourced<T: VromProxy + Copy> {
    vrom_end: VromAddr,
    value: T,
}

impl<T: VromProxy + Copy> RangeSourced<T> {
    pub fn from_vrom_range(
        vrom: Vrom<'_>,
        vrom_range: Range<VromAddr>,
    ) -> Result<RangeSourced<T>, ReadError> {
        Ok(RangeSourced {
            vrom_end: vrom_range.end,
            value: T::from_vrom(vrom, vrom_range.start)?,
        })
    }

    pub fn vrom_range(&self) -> Range<VromAddr> {
        self.addr()..self.vrom_end
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: VromProxy + Copy> Deref for RangeSourced<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}
