use byteorder::{BigEndian, ReadBytesExt};
use oot_explorer_rom::{Rom, RomAddr, RomError};
use std::ops::Range;

use crate::VromAddr;

#[derive(Clone)]
pub struct FileSystemTableEntry {
    pub virtual_start: VromAddr,
    pub virtual_end: VromAddr,
    pub physical_start: RomAddr,
    pub physical_end: RomAddr,
}

impl FileSystemTableEntry {
    pub const SIZE: u32 = 16;

    pub fn from_rom(rom: Rom<'_>, addr: RomAddr) -> Result<Self, RomError> {
        let mut data = rom.slice(addr..addr + Self::SIZE)?;

        let virtual_start = VromAddr(data.read_u32::<BigEndian>().unwrap());
        let virtual_end = VromAddr(data.read_u32::<BigEndian>().unwrap());
        let physical_start = RomAddr(data.read_u32::<BigEndian>().unwrap());
        let physical_end = RomAddr(data.read_u32::<BigEndian>().unwrap());

        Ok(Self {
            virtual_start,
            virtual_end,
            physical_start,
            physical_end,
        })
    }

    pub fn virtual_range(&self) -> Range<VromAddr> {
        self.virtual_start..self.virtual_end
    }

    pub fn physical_range(&self) -> Range<RomAddr> {
        self.physical_start..self.physical_end
    }

    pub fn is_end(&self) -> bool {
        self.virtual_range() == (VromAddr(0)..VromAddr(0))
    }

    pub fn is_present(&self) -> bool {
        self.physical_range() != (RomAddr(0xffffffff)..RomAddr(0xffffffff))
    }

    pub fn is_compressed(&self) -> bool {
        self.physical_end.0 > 0
    }
}
