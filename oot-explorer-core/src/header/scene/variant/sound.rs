use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct SceneSoundHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for SceneSoundHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SceneSoundHeader")
            .field("settings", &self.settings())
            .field("night_sound", &self.night_sound())
            .field("music", &self.music())
            .finish()
    }
}

impl<'a> SceneSoundHeader<'a> {
    pub fn new(data: &'a [u8]) -> SceneSoundHeader<'a> {
        SceneSoundHeader { data }
    }

    pub fn settings(self) -> u8 {
        self.data[1]
    }

    pub fn night_sound(self) -> u8 {
        self.data[6]
    }

    pub fn music(self) -> u8 {
        self.data[7]
    }
}
