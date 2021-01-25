use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::reflect::dump;
use crate::reflect::instantiate::Instantiate;
use crate::reflect::type_::TypeDescriptor;
use crate::segment::{SegmentAddr, SegmentCtx, SegmentResolveError};

pub struct PointerDescriptor {
    pub name: &'static str,
    pub target: TypeDescriptor,
}

pub(super) fn dump_pointer<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    indent_level: usize,
    desc: &'static PointerDescriptor,
    addr: VromAddr,
) {
    let segment_addr = match fs.get_virtual_slice(scope, addr..addr + 4) {
        Ok(data) => <SegmentAddr as Instantiate>::new(data),
        Err(VirtualSliceError::OutOfRange { .. }) => {
            print!("(inaccessible)");
            return;
        }
    };

    let addr = match segment_ctx.resolve_vrom(segment_addr) {
        Ok(range) => range.start,
        Err(SegmentResolveError::Unmapped { segment }) => {
            print!("(segment 0x{:x} not mapped)", segment.0);
            return;
        }
    };

    print!("&");
    dump(scope, fs, segment_ctx, indent_level, desc.target, addr)
}
