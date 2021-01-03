use crate::fs::VromAddr;
use crate::header::{self, RoomHeader};
use std::ops::Range;

#[derive(Clone, Copy)]
pub struct Room<'a> {
    addr: VromAddr,
    data: &'a [u8],
}
impl<'a> Room<'a> {
    pub fn new(addr: VromAddr, data: &'a [u8]) -> Room<'a> {
        Room { addr, data }
    }
    pub fn addr(self) -> VromAddr {
        self.addr
    }
    pub fn vrom_range(self) -> Range<VromAddr> {
        self.addr..(self.addr + self.data.len() as u32)
    }
    pub fn data(self) -> &'a [u8] {
        self.data
    }
    pub fn headers(self) -> impl Iterator<Item = RoomHeader<'a>> {
        header::Iter::new(self.data).map(|header| header.room_header())
    }
}
