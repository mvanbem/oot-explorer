use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct RoomSkyboxHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for RoomSkyboxHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomSkyboxHeader")
            .field("disable_sky", &self.disable_sky())
            .field("disable_sun_moon", &self.disable_sun_moon())
            .finish()
    }
}

impl<'a> RoomSkyboxHeader<'a> {
    pub fn new(data: &'a [u8]) -> RoomSkyboxHeader<'a> {
        RoomSkyboxHeader { data }
    }

    pub fn disable_sky(self) -> bool {
        self.data[4] != 0
    }

    pub fn disable_sun_moon(self) -> bool {
        self.data[5] != 0
    }
}
