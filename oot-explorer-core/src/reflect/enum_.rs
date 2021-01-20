use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::reflect::primitive::PrimitiveType;

pub struct EnumDescriptor {
    pub name: &'static str,
    pub underlying: PrimitiveType,
    pub values: &'static [(u32, &'static str)],
}

impl EnumDescriptor {
    pub fn read_as_u32<'scope>(
        &self,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        addr: VromAddr,
    ) -> Result<u32, VirtualSliceError> {
        self.underlying.read_as_u32(scope, fs, addr)
    }
}

pub(super) fn dump_enum<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    desc: &'static EnumDescriptor,
    addr: VromAddr,
) {
    match desc.underlying.read_as_u32(scope, fs, addr) {
        Ok(value) => match desc.values.binary_search_by_key(&value, |&(x, _)| x) {
            Ok(index) => print!("{}", desc.values[index].1),
            Err(_) => print!("(unknown value 0x{:x}", value),
        },
        Err(VirtualSliceError::OutOfRange { .. }) => print!("(inaccessible)"),
    }
}
