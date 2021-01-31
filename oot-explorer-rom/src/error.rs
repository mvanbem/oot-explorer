use thiserror::Error;

use crate::RomAddr;

#[derive(Debug, Error)]
pub enum RomError {
    #[error("ROM access out of range: {from:?}..{to:?}, ROM size {rom_size:08x}")]
    OutOfRange {
        from: Option<RomAddr>,
        to: Option<RomAddr>,
        rom_size: u32,
    },
}
