use oot_explorer_read::ReadError;
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::{
    BitfieldDescriptor, EnumDescriptor, IsEndFn, PointerDescriptor, PrimitiveType,
    StructDescriptor, UnionDescriptor,
};

#[derive(Clone, Copy)]
pub enum TypeDescriptor {
    Struct(&'static StructDescriptor),
    Union(&'static UnionDescriptor),
    Enum(&'static EnumDescriptor),
    Bitfield(&'static BitfieldDescriptor),
    Primitive(PrimitiveType),
    Pointer(&'static PointerDescriptor),
}

impl TypeDescriptor {
    pub fn name(&self) -> &'static str {
        match self {
            TypeDescriptor::Struct(desc) => desc.name,
            TypeDescriptor::Union(desc) => desc.name,
            TypeDescriptor::Enum(desc) => desc.name,
            TypeDescriptor::Bitfield(desc) => desc.name,
            TypeDescriptor::Primitive(desc) => desc.name(),
            TypeDescriptor::Pointer(desc) => desc.name,
        }
    }

    pub fn size(&self) -> Option<u32> {
        match self {
            TypeDescriptor::Struct(desc) => desc.size,
            TypeDescriptor::Union(desc) => desc.size,
            TypeDescriptor::Enum(_) => None,
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(desc) => Some(desc.size()),
            TypeDescriptor::Pointer(_) => Some(4),
        }
    }

    pub fn is_end(&self) -> Option<IsEndFn> {
        match self {
            TypeDescriptor::Struct(desc) => desc.is_end,
            TypeDescriptor::Union(desc) => desc.is_end,
            TypeDescriptor::Enum(_) => None,
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(_) => None,
            TypeDescriptor::Pointer(_) => None,
        }
    }

    pub fn read_as_u32(&self, vrom: Vrom<'_>, addr: VromAddr) -> Option<Result<u32, ReadError>> {
        match self {
            TypeDescriptor::Struct(_) => None,
            TypeDescriptor::Union(_) => None,
            TypeDescriptor::Enum(desc) => Some(desc.read_as_u32(vrom, addr)),
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(desc) => Some(desc.read_as_u32(vrom, addr)),
            TypeDescriptor::Pointer(_) => None,
        }
    }
}
