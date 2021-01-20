use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::type_::TypeDescriptor;

pub const ELF_MESSAGE_DESC: TypeDescriptor = TypeDescriptor::Enum(&EnumDescriptor {
    name: "ElfMessage",
    underlying: PrimitiveType::U8,
    values: &[(0x00, "NONE"), (0x01, "FIELD"), (0x02, "YDAN")],
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfMessage {
    None,
    Field,
    Ydan,
}

impl ElfMessage {
    pub fn parse(value: u8) -> ElfMessage {
        match value {
            0x00 => ElfMessage::None,
            0x01 => ElfMessage::Field,
            0x02 => ElfMessage::Ydan,
            _ => panic!("unexpected elf message value: 0x{:02x}", value),
        }
    }
}

impl<'scope> Instantiate<'scope> for ElfMessage {
    fn new(data: &'scope [u8]) -> Self {
        ElfMessage::parse(data[0])
    }
}

impl ReflectSized for ElfMessage {
    const SIZE: usize = 1;
}
