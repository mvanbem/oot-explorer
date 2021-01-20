use std::ops::{Deref, Range};

use crate::fs::VromAddr;
use crate::reflect::sized::ReflectSized;

#[derive(Clone)]
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
}

impl<'scope, T> Sourced<T>
where
    T: ReflectSized,
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

#[derive(Clone)]
pub struct RangeSourced<T> {
    vrom_range: Range<VromAddr>,
    value: T,
}

impl<T> RangeSourced<T> {
    pub fn new(vrom_range: Range<VromAddr>, value: T) -> RangeSourced<T> {
        RangeSourced { vrom_range, value }
    }

    pub fn addr(&self) -> VromAddr {
        self.vrom_range.start
    }

    pub fn vrom_range(&self) -> Range<VromAddr> {
        self.vrom_range.clone()
    }
}

impl<T> Deref for RangeSourced<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}
