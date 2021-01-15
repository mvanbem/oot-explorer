use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct RoomBehaviorHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for RoomBehaviorHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RoomBehaviorHeader")
            .field("x", &self.x())
            .field("disable_warp_songs", &self.disable_warp_songs())
            .field("show_invisible_actors", &self.show_invisible_actors())
            .field("idle_animation_or_heat", &self.idle_animation_or_heat())
            .finish()
    }
}

impl<'a> RoomBehaviorHeader<'a> {
    pub fn new(data: &'a [u8]) -> RoomBehaviorHeader<'a> {
        RoomBehaviorHeader { data }
    }

    /// Affects Sun's Song, backflipping with A.
    pub fn x(self) -> u8 {
        self.data[1]
    }

    pub fn disable_warp_songs(self) -> u8 {
        self.data[6] >> 4
    }

    pub fn show_invisible_actors(self) -> bool {
        self.data[6] & 1 == 1
    }

    pub fn idle_animation_or_heat(self) -> u8 {
        self.data[7]
    }
}
