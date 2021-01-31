use oot_explorer_read::{FromVrom, ReadError};
use oot_explorer_vrom::{Vrom, VromAddr};

#[derive(Clone, Copy)]
pub enum PrimitiveType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    VromAddr,
    SegmentAddr,
}

impl PrimitiveType {
    pub fn name(self) -> &'static str {
        match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "u8",
            PrimitiveType::I8 => "i8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::I16 => "i16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::I32 => "i32",
            PrimitiveType::VromAddr => "VromAddr",
            PrimitiveType::SegmentAddr => "SegmentAddr",
        }
    }

    pub fn size(self) -> u32 {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::I8 => 1,
            PrimitiveType::U16 | PrimitiveType::I16 => 2,
            PrimitiveType::U32
            | PrimitiveType::I32
            | PrimitiveType::VromAddr
            | PrimitiveType::SegmentAddr => 4,
        }
    }

    pub fn read_as_u32(self, vrom: Vrom<'_>, addr: VromAddr) -> Result<u32, ReadError> {
        Ok(match self {
            PrimitiveType::Bool => bool::from_vrom(vrom, addr)? as u32,
            PrimitiveType::U8 => u8::from_vrom(vrom, addr)? as u32,
            PrimitiveType::I8 => i8::from_vrom(vrom, addr)? as u32,
            PrimitiveType::U16 => u16::from_vrom(vrom, addr)? as u32,
            PrimitiveType::I16 => i16::from_vrom(vrom, addr)? as u32,
            PrimitiveType::U32 | PrimitiveType::VromAddr | PrimitiveType::SegmentAddr => {
                u32::from_vrom(vrom, addr)?
            }
            PrimitiveType::I32 => i32::from_vrom(vrom, addr)? as u32,
        })
    }
}
