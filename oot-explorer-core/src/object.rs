use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::type_::TypeDescriptor;

pub const OBJECT_ID_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U16);

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(pub u16);

impl<'a> Instantiate<'a> for ObjectId {
    fn new(data: &'a [u8]) -> ObjectId {
        ObjectId(<u16 as Instantiate>::new(data))
    }
}

impl<'a> ReflectSized for ObjectId {
    const SIZE: usize = 2;
}
