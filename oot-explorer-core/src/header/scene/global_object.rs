use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::type_::TypeDescriptor;

pub const GLOBAL_OBJECT_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "GlobalObject",
    underlying: PrimitiveType::U16,
    values: &[(0x0000, "NONE"), (0x0002, "FIELD"), (0x0003, "DANGEON")],
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalObject {
    None,
    Field,
    Dangeon,
}

impl GlobalObject {
    pub fn parse(value: u16) -> GlobalObject {
        match value {
            0x0000 => GlobalObject::None,
            0x0002 => GlobalObject::Field,
            0x0003 => GlobalObject::Dangeon,
            _ => panic!("unexpected global object value: 0x{:04x}", value),
        }
    }
}

impl<'scope> Instantiate<'scope> for GlobalObject {
    fn new(data: &'scope [u8]) -> Self {
        GlobalObject::parse(<u16 as Instantiate>::new(data))
    }
}

impl ReflectSized for GlobalObject {
    const SIZE: usize = 1;
}
