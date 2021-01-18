use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::reflect::enum_::EnumDescriptor;
use crate::reflect::primitive::PrimitiveType;

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

pub(super) fn dump_bitfield<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    desc: &'static BitfieldDescriptor,
    addr: VromAddr,
) -> () {
    let value = match desc.underlying.read_as_u32(scope, fs, addr) {
        Ok(value) => value,
        Err(VirtualSliceError::OutOfRange { .. }) => {
            print!("(inaccessible)");
            return;
        }
    };

    let mut first = true;
    for field in desc.fields {
        if first {
            first = false;
        } else {
            print!(" | ");
        }

        let value = (value >> field.shift) & field.mask;
        // TODO: How do we dump a value that doesn't exist in VROM? Does we need a dump_value that
        // can only forward to enum and primitive?
        print!("{}", value);
    }
}
