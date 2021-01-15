use crate::header::room::type_::RoomHeaderType;
use crate::header::room::RoomHeader;

pub struct RoomHeaderIter<'a> {
    data: &'a [u8],
}

impl<'a> RoomHeaderIter<'a> {
    pub fn new(data: &'a [u8]) -> RoomHeaderIter<'a> {
        RoomHeaderIter { data }
    }
}

impl<'a> Iterator for RoomHeaderIter<'a> {
    type Item = RoomHeader<'a>;

    fn next(&mut self) -> Option<RoomHeader<'a>> {
        let header = RoomHeader::new(self.data);
        if header.type_() == RoomHeaderType::END {
            None
        } else {
            self.data = &self.data[RoomHeader::SIZE..];
            Some(header)
        }
    }
}
