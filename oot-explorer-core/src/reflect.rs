use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VromAddr};
use crate::reflect::bitfield::dump_bitfield;
use crate::reflect::enum_::dump_enum;
use crate::reflect::primitive::dump_primitive;
use crate::reflect::struct_::{dump_struct, dump_union};
use crate::reflect::type_::TypeDescriptor;
use crate::segment::SegmentCtx;

pub mod bitfield;
pub mod enum_;
pub mod instantiate;
pub mod primitive;
pub mod sized;
pub mod sourced;
pub mod struct_;
pub mod type_;

pub fn dump<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    indent_level: usize,
    desc: TypeDescriptor,
    addr: VromAddr,
) {
    match desc {
        TypeDescriptor::Struct(desc) => {
            dump_struct(scope, fs, segment_ctx, indent_level, desc, addr);
        }
        TypeDescriptor::Union(desc) => {
            dump_union(scope, fs, segment_ctx, indent_level, desc, addr);
        }
        TypeDescriptor::Enum(desc) => dump_enum(scope, fs, desc, addr),
        TypeDescriptor::Bitfield(desc) => dump_bitfield(scope, fs, desc, addr),
        TypeDescriptor::Primitive(desc) => dump_primitive(scope, fs, desc, addr),
    }
}
