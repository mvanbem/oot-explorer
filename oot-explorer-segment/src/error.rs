use oot_explorer_vrom::VromError;
use thiserror::Error;

use crate::Segment;

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("{0}")]
    VromError(#[from] VromError),

    #[error("segment out of range: {0:?}")]
    BadSegment(Segment),

    #[error("unmapped segment: {0:?}")]
    Unmapped(Segment),
}
