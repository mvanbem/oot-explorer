use oot_explorer_segment::SegmentError;
use oot_explorer_vrom::{GetFileError, VromAddr, VromError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("{0}")]
    GetFileError(#[from] GetFileError),

    #[error("{0}")]
    VromError(#[from] VromError),

    #[error("{0}")]
    SegmentError(#[from] SegmentError),

    #[error("misaligned access: need {align_bits} trailing zero bits in {addr:?}")]
    Misaligned { align_bits: u32, addr: VromAddr },
}
