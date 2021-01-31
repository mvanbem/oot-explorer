use oot_explorer_vrom::{Vrom, VromAddr};

use crate::{PrimitiveType, TypeDescriptor};

pub struct StructDescriptor {
    pub name: &'static str,
    pub size: Option<u32>,
    pub is_end: Option<IsEndFn>,
    pub fields: &'static [FieldDescriptor],
}

pub type IsEndFn = fn(Vrom<'_>, VromAddr) -> bool;

pub struct FieldDescriptor {
    pub name: &'static str,
    pub location: StructFieldLocation,
    pub desc: TypeDescriptor,
}

#[derive(Clone)]
pub enum StructFieldLocation {
    Simple {
        offset: u32,
    },
    Slice {
        count_offset: u32,
        count_desc: PrimitiveType,
        ptr_offset: u32,
    },
    InlineDelimitedList {
        offset: u32,
    },
}

pub struct UnionDescriptor {
    pub name: &'static str,
    pub size: Option<u32>,
    pub is_end: Option<IsEndFn>,
    pub discriminant_offset: u32,
    pub discriminant_desc: TypeDescriptor,
    pub variants: &'static [(u32, TypeDescriptor)],
}
