use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VromAddr};
use crate::reflect::bitfield::BitfieldDescriptor;
use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::struct_::{IsEndFn, StructDescriptor, UnionDescriptor};

#[derive(Clone, Copy)]
pub enum TypeDescriptor {
    Struct(&'static StructDescriptor),
    Union(&'static UnionDescriptor),
    Enum(&'static EnumDescriptor),
    Bitfield(&'static BitfieldDescriptor),
    Primitive(PrimitiveType),
}

impl TypeDescriptor {
    pub fn name(&self) -> &'static str {
        match self {
            TypeDescriptor::Struct(desc) => desc.name,
            TypeDescriptor::Union(desc) => desc.name,
            TypeDescriptor::Enum(desc) => desc.name,
            TypeDescriptor::Bitfield(desc) => desc.name,
            TypeDescriptor::Primitive(desc) => desc.name(),
        }
    }

    pub fn size(&self) -> Option<u32> {
        match self {
            TypeDescriptor::Struct(desc) => desc.size,
            TypeDescriptor::Union(desc) => desc.size,
            TypeDescriptor::Enum(_) => None,
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(_) => None,
        }
    }

    pub fn is_end(&self) -> Option<IsEndFn> {
        match self {
            TypeDescriptor::Struct(desc) => desc.is_end,
            TypeDescriptor::Union(desc) => desc.is_end,
            TypeDescriptor::Enum(_) => None,
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(_) => None,
        }
    }

    pub fn read_as_u32<'scope>(
        &self,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        addr: VromAddr,
    ) -> Option<u32> {
        match self {
            TypeDescriptor::Struct(_) => None,
            TypeDescriptor::Union(_) => None,
            TypeDescriptor::Enum(desc) => desc.read_as_u32(scope, fs, addr).ok(),
            TypeDescriptor::Bitfield(_) => None,
            TypeDescriptor::Primitive(desc) => desc.read_as_u32(scope, fs, addr).ok(),
        }
    }
}
