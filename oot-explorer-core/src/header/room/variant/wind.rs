use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct WindHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for WindHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("WindHeader")
            .field("west", &self.west())
            .field("up", &self.up())
            .field("south", &self.south())
            .field("strength", &self.strength())
            .finish()
    }
}

impl<'a> WindHeader<'a> {
    pub fn new(data: &'a [u8]) -> WindHeader<'a> {
        WindHeader { data }
    }

    pub fn west(self) -> i8 {
        self.data[4] as i8
    }

    pub fn up(self) -> i8 {
        self.data[5] as i8
    }

    pub fn south(self) -> i8 {
        self.data[6] as i8
    }

    pub fn strength(self) -> u8 {
        self.data[7]
    }
}
