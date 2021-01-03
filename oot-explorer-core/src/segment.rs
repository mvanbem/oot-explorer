use crate::fs::VromAddr;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, Range, Sub};
use thiserror::Error;

/// A segmented address.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct SegmentAddr(pub u32);
impl Debug for SegmentAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (segment, offset) = (self.segment(), self.offset());
        match segment {
            Segment::SCENE => write!(f, "CurrentScene(0x{:06x})", offset),
            Segment::ROOM => write!(f, "CurrentRoom(0x{:06x})", offset),
            Segment::GAMEPLAY_KEEP => write!(f, "GameplayKeep(0x{:06x})", offset),
            Segment::SELECTABLE_KEEP => write!(f, "SelectableKeep(0x{:06x})", offset),
            Segment::OBJECT => write!(f, "CurrentObject(0x{:06x})", offset),
            _ => write!(f, "UnknownSegment(0x{:02x}, 0x{:06x})", segment.0, offset),
        }
    }
}
impl SegmentAddr {
    pub fn segment(self) -> Segment {
        Segment((self.0 >> 24) as u8)
    }
    pub fn offset(self) -> u32 {
        self.0 & 0x00ff_ffff
    }
}
impl Add<u32> for SegmentAddr {
    type Output = SegmentAddr;
    fn add(self, rhs: u32) -> SegmentAddr {
        let result = SegmentAddr(self.0 + rhs);
        assert_eq!(result.segment(), self.segment());
        result
    }
}
impl Sub for SegmentAddr {
    type Output = u32;
    fn sub(self, rhs: SegmentAddr) -> u32 {
        assert_eq!(self.segment(), rhs.segment());
        self.0 - rhs.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Segment(pub u8);
impl Segment {
    pub const SCENE: Segment = Segment(0x02);
    pub const ROOM: Segment = Segment(0x03);
    pub const GAMEPLAY_KEEP: Segment = Segment(0x04);
    pub const SELECTABLE_KEEP: Segment = Segment(0x05);
    pub const OBJECT: Segment = Segment(0x06);
}

#[derive(Clone)]
pub struct SegmentCtx<'a> {
    mappings: HashMap<Segment, (&'a [u8], Range<VromAddr>)>,
}
impl<'a> SegmentCtx<'a> {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
    pub fn set(&mut self, segment: Segment, data: &'a [u8], vrom: Range<VromAddr>) {
        self.mappings.insert(segment, (data, vrom));
    }
    pub fn get_data(&self, segment: Segment) -> Option<&'a [u8]> {
        self.mappings.get(&segment).map(|tuple| tuple.0)
    }
    pub fn get_vrom(&self, segment: Segment) -> Option<Range<VromAddr>> {
        self.mappings.get(&segment).map(|tuple| tuple.1.clone())
    }

    pub fn resolve(&self, addr: SegmentAddr) -> Result<&'a [u8], SegmentResolveError> {
        if let Some((data, _)) = self.mappings.get(&addr.segment()) {
            return Ok(&data[addr.offset() as usize..]);
        }
        Err(SegmentResolveError::Unmapped {
            segment: addr.segment(),
        })
    }
    pub fn resolve_range(
        &self,
        range: Range<SegmentAddr>,
    ) -> Result<&'a [u8], SegmentResolveError> {
        // TODO: Maybe this slice operation shouldn't be permitted to panic to caller. As far as
        // I've seen, either the segment is mapped or it isn't, but this could be a bad range in a
        // mapped segment.
        self.resolve(range.start)
            .map(|data| &data[..(range.end - range.start) as usize])
    }
    pub fn resolve_vrom(&self, addr: SegmentAddr) -> Result<Range<VromAddr>, SegmentResolveError> {
        if let Some((_, ref vrom)) = self.mappings.get(&addr.segment()) {
            return Ok(vrom.start + addr.offset()..vrom.end);
        }
        Err(SegmentResolveError::Unmapped {
            segment: addr.segment(),
        })
    }
}

#[derive(Debug, Error)]
pub enum SegmentResolveError {
    #[error("unmapped segment: {segment:?}")]
    Unmapped { segment: Segment },
}
