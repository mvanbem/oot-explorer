use std::fmt::{self, Debug, Formatter};

use crate::header::scene::variant::camera::camera::Camera;

pub mod camera;

#[derive(Clone, Copy)]
pub struct CameraAndWorldMapHeader<'a> {
    data: &'a [u8],
}

impl<'a> Debug for CameraAndWorldMapHeader<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CameraAndWorldMapHeader")
            .field("camera", &self.camera())
            .field("world_map_location", &self.world_map_location())
            .finish()
    }
}

impl<'a> CameraAndWorldMapHeader<'a> {
    pub fn new(data: &'a [u8]) -> CameraAndWorldMapHeader<'a> {
        CameraAndWorldMapHeader { data }
    }

    // Raw getters
    pub fn raw_camera(self) -> u8 {
        self.data[1]
    }

    pub fn world_map_location(self) -> u8 {
        self.data[7]
    }

    // Interpreted getters
    pub fn camera(self) -> Camera {
        Camera::parse(self.raw_camera())
    }
}
