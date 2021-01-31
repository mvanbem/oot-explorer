use std::fmt::{self, Debug, Formatter};

use crate::SegmentError;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Segment(pub u8);

impl Segment {
    pub const SCENE: Segment = Segment(0x02);
    pub const ROOM: Segment = Segment(0x03);
    pub const GAMEPLAY_KEEP: Segment = Segment(0x04);
    pub const SELECTABLE_KEEP: Segment = Segment(0x05);
    pub const OBJECT: Segment = Segment(0x06);

    pub fn validate(self) -> Result<Self, SegmentError> {
        if self.0 & 0x0f == self.0 {
            Ok(self)
        } else {
            Err(SegmentError::BadSegment(self))
        }
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Segment::SCENE => write!(f, "CurrentScene"),
            Segment::ROOM => write!(f, "CurrentRoom"),
            Segment::GAMEPLAY_KEEP => write!(f, "GameplayKeep"),
            Segment::SELECTABLE_KEEP => write!(f, "SelectableKeep"),
            Segment::OBJECT => write!(f, "CurrentObject"),
            _ => write!(f, "Unknown(0x{:02x})", self.0),
        }
    }
}
