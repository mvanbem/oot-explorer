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
