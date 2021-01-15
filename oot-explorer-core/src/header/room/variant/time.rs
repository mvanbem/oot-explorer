use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct TimeHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for TimeHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TimeHeader")
            .field("time_override", &self.time_override())
            .field("time_speed", &self.time_speed())
            .finish()
    }
}

impl<'a> TimeHeader<'a> {
    pub fn new(data: &'a [u8]) -> TimeHeader<'a> {
        TimeHeader { data }
    }

    pub fn time_override(self) -> Option<u16> {
        let time = (&self.data[4..]).read_u16::<BigEndian>().unwrap();
        if time == 0xffff {
            None
        } else {
            Some(time)
        }
    }

    pub fn time_speed(self) -> i8 {
        self.data[6] as i8
    }
}
