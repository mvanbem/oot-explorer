use oot_explorer_vrom::VromAddr;

use crate::{Segment, SegmentAddr, SegmentError};

const INVALID_BASE_ADDR: VromAddr = VromAddr(0xffff_ffff);

#[derive(Clone)]
pub struct SegmentTable {
    base_addrs: [VromAddr; 16],
}

impl SegmentTable {
    pub fn new() -> Self {
        Self {
            base_addrs: [INVALID_BASE_ADDR; 16],
        }
    }

    pub fn set(&mut self, segment: Segment, addr: VromAddr) -> Result<(), SegmentError> {
        self.base_addrs[segment.validate()?.0 as usize] = addr;
        Ok(())
    }

    pub fn with(&self, segment: Segment, addr: VromAddr) -> Result<SegmentTable, SegmentError> {
        let mut segment_table = self.clone();
        segment_table.set(segment, addr)?;
        Ok(segment_table)
    }

    pub fn get(&self, segment: Segment) -> Result<VromAddr, SegmentError> {
        match self.base_addrs[segment.validate()?.0 as usize] {
            INVALID_BASE_ADDR => Err(SegmentError::Unmapped(segment)),
            addr => Ok(addr),
        }
    }

    pub fn resolve(&self, segment_addr: SegmentAddr) -> Result<VromAddr, SegmentError> {
        let segment = segment_addr.segment().validate()?;
        match self.base_addrs[segment.0 as usize] {
            INVALID_BASE_ADDR => Err(SegmentError::Unmapped(segment)),
            base_addr => Ok(base_addr.checked_add(segment_addr.offset())?),
        }
    }
}

impl Default for SegmentTable {
    fn default() -> Self {
        Self::new()
    }
}
