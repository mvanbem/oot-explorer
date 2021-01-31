use crate::TypeDescriptor;

pub struct PointerDescriptor {
    pub name: &'static str,
    pub target: TypeDescriptor,
}
