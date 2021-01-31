use crate::{EnumDescriptor, PrimitiveType};

pub struct BitfieldDescriptor {
    pub name: &'static str,
    pub underlying: PrimitiveType,
    pub fields: &'static [BitfieldSpan],
}

pub struct BitfieldSpan {
    /// Shifting is applied before masking.
    pub shift: u8,
    /// Masking is applied after shifting.
    pub mask: u32,
    pub desc: &'static EnumDescriptor,
}
