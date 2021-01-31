use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, Range};

use crate::{VromAddr, VromError};

/// A slice representing all of VROM.
#[derive(Clone, Copy)]
pub struct Vrom<'a>(pub &'a [u8]);

impl<'a> Vrom<'a> {
    pub fn slice_from(self, from: VromAddr) -> Result<&'a [u8], VromError> {
        self.0
            .get(from.0 as usize..)
            .ok_or_else(|| VromError::OutOfRange {
                from: Some(from),
                to: None,
                vrom_size: self.0.len() as u32,
            })
    }

    pub fn slice_to(self, to: VromAddr) -> Result<&'a [u8], VromError> {
        self.0
            .get(..to.0 as usize)
            .ok_or_else(|| VromError::OutOfRange {
                from: None,
                to: Some(to),
                vrom_size: self.0.len() as u32,
            })
    }

    pub fn slice(self, range: Range<VromAddr>) -> Result<&'a [u8], VromError> {
        self.0
            .get(range.start.0 as usize..range.end.0 as usize)
            .ok_or_else(|| VromError::OutOfRange {
                from: Some(range.start),
                to: Some(range.end),
                vrom_size: self.0.len() as u32,
            })
    }
}

impl<'a> Debug for Vrom<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vrom(_)")
    }
}

impl<'a> Deref for Vrom<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}
