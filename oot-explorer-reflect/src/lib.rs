mod bitfield;
mod enum_;
mod pointer;
mod primitive;
mod sourced;
mod struct_;
mod type_;

pub use bitfield::{BitfieldDescriptor, BitfieldSpan};
pub use enum_::EnumDescriptor;
pub use pointer::PointerDescriptor;
pub use primitive::PrimitiveType;
pub use sourced::{RangeSourced, Sourced};
pub use struct_::{
    FieldDescriptor, IsEndFn, StructDescriptor, StructFieldLocation, UnionDescriptor,
};
pub use type_::TypeDescriptor;

pub const BOOL_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::Bool);
pub const U8_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U8);
pub const I8_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I8);
pub const U16_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U16);
pub const I16_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I16);
pub const U32_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U32);
pub const I32_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I32);
pub const VROM_ADDR_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::VromAddr);
pub const SEGMENT_ADDR_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::SegmentAddr);
