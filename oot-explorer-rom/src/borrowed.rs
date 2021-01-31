use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, Range};

use crate::{RomAddr, RomError};

/// A slice representing all of ROM.
#[derive(Clone, Copy)]
pub struct Rom<'a>(pub &'a [u8]);

impl<'a> Rom<'a> {
    pub fn slice_from(self, from: RomAddr) -> Result<&'a [u8], RomError> {
        self.0
            .get(from.0 as usize..)
            .ok_or_else(|| RomError::OutOfRange {
                from: Some(from),
                to: None,
                rom_size: self.0.len() as u32,
            })
    }

    pub fn slice_to(self, to: RomAddr) -> Result<&'a [u8], RomError> {
        self.0
            .get(..to.0 as usize)
            .ok_or_else(|| RomError::OutOfRange {
                from: None,
                to: Some(to),
                rom_size: self.0.len() as u32,
            })
    }

    pub fn slice(self, range: Range<RomAddr>) -> Result<&'a [u8], RomError> {
        self.0
            .get(range.start.0 as usize..range.end.0 as usize)
            .ok_or_else(|| RomError::OutOfRange {
                from: Some(range.start),
                to: Some(range.end),
                rom_size: self.0.len() as u32,
            })
    }
}

impl<'a> Debug for Rom<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Rom(_)")
    }
}

impl<'a> Deref for Rom<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}
