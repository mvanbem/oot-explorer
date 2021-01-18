use std::ops::Deref;

use crate::fs::VromAddr;

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

impl<T> Deref for Sourced<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}
