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
