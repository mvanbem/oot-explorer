use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct SceneSkyboxHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for SceneSkyboxHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("SceneSkyboxHeader")
            .field("skybox", &self.skybox())
            .field("cloudy", &self.cloudy())
            .field("indoor_lighting", &self.indoor_lighting())
            .finish()
    }
}

impl<'a> SceneSkyboxHeader<'a> {
    pub fn new(data: &'a [u8]) -> SceneSkyboxHeader<'a> {
        SceneSkyboxHeader { data }
    }

    pub fn skybox(self) -> u8 {
        self.data[4]
    }

    pub fn cloudy(self) -> bool {
        self.data[5] != 0
    }

    pub fn indoor_lighting(self) -> bool {
        self.data[6] != 0
    }
}
