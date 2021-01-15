#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Camera {
    Free,
    FixedWithAlternate,
    RotateWithAlternate,
    Fixed,
    Rotate,
    ShootingGallery,
}

impl Camera {
    pub fn parse(value: u8) -> Camera {
        match value {
            0x00 => Camera::Free,
            0x10 => Camera::FixedWithAlternate,
            0x20 => Camera::RotateWithAlternate,
            0x30 => Camera::Fixed,
            0x40 => Camera::Rotate,
            0x50 => Camera::ShootingGallery,
            _ => panic!("unexpected camera value: 0x{:02x}", value),
        }
    }
}
