use thiserror::Error;

use crate::VromAddr;

#[derive(Debug, Error)]
pub enum VromError {
    #[error("VROM access out of range: {from:?}..{to:?}, VROM size {vrom_size:08x}")]
    OutOfRange {
        from: Option<VromAddr>,
        to: Option<VromAddr>,
        vrom_size: u32,
    },

    #[error("VROM address overflow: {addr:?} + {offset:08x}")]
    VromAddrOverflow { addr: VromAddr, offset: u32 },
}
