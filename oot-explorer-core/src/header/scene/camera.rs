use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::type_::TypeDescriptor;

pub const CAMERA_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "Camera",
    underlying: PrimitiveType::U8,
    values: &[
        (0x00, "FREE"),
        (0x10, "FIXED_WITH_ALTERNATE"),
        (0x20, "ROTATE_WITH_ALTERNATE"),
        (0x30, "FIXED"),
        (0x40, "ROTATE"),
        (0x50, "SHOOTING_GALLERY"),
    ],
});

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

impl<'scope> Instantiate<'scope> for Camera {
    fn new(data: &'scope [u8]) -> Self {
        Camera::parse(<u8 as Instantiate>::new(data))
    }
}

impl ReflectSized for Camera {
    const SIZE: usize = 1;
}
