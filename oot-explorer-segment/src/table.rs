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

    /// Sets a table entry.
    ///
    /// # Panics
    ///
    /// Panics if `segment` is greater than 15.
    pub fn set(&mut self, segment: Segment, addr: VromAddr) {
        self.base_addrs[segment.validate().unwrap().0 as usize] = addr;
    }

    /// Returns a copy of this table with one entry modified.
    ///
    /// # Panics
    ///
    /// Panics if `segment` is greater than 15.
    pub fn with(&self, segment: Segment, addr: VromAddr) -> Self {
        let mut result = self.clone();
        result.set(segment, addr);
        result
    }

    /// Gets a table entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested segment is unmapped.
    ///
    /// # Panics
    ///
    /// Panics if `segment` is greater than 15.
    pub fn get(&self, segment: Segment) -> Result<VromAddr, SegmentError> {
        match self.base_addrs[segment.validate().unwrap().0 as usize] {
            INVALID_BASE_ADDR => Err(SegmentError::Unmapped(segment)),
            addr => Ok(addr),
        }
    }

    /// Resolves a segment address.
    ///
    /// This method uses only the lower four bits of the segment, ignoring the upper four bits.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested segment is unmapped.
    pub fn resolve(&self, segment_addr: SegmentAddr) -> Result<VromAddr, SegmentError> {
        let segment = segment_addr.segment().masked();
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
