use oot_explorer_read::{FromVrom, Layout, ReadError};
use oot_explorer_reflect::{PrimitiveType, TypeDescriptor};
use oot_explorer_vrom::{Vrom, VromAddr};

pub const OBJECT_ID_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U16);

// TODO: Codegen for newtype wrappers.

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(pub u16);

impl FromVrom for ObjectId {
    fn from_vrom(vrom: Vrom<'_>, addr: VromAddr) -> Result<Self, ReadError> {
        Ok(Self(<u16 as FromVrom>::from_vrom(vrom, addr)?))
    }
}

impl Layout for ObjectId {
    const SIZE: u32 = 2;
}
