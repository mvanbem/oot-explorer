use byteorder::{BigEndian, ReadBytesExt};
use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VromAddr};
use crate::reflect::dump;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::type_::TypeDescriptor;
use crate::segment::{SegmentAddr, SegmentCtx, SegmentResolveError};

pub struct StructDescriptor {
    pub name: &'static str,
    pub size: Option<u32>,
    pub is_end: Option<IsEndFn>,
    pub fields: &'static [FieldDescriptor],
}

pub type IsEndFn =
    for<'scope> fn(&'scope ScopedOwner, &mut LazyFileSystem<'scope>, VromAddr) -> bool;

pub struct FieldDescriptor {
    pub name: &'static str,
    pub location: StructFieldLocation,
    pub desc: TypeDescriptor,
}

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
    pub variants: &'static [Option<&'static VariantDescriptor>],
}

pub struct VariantDescriptor {
    pub fields: &'static [FieldDescriptor],
}

pub(super) fn dump_struct<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    indent_level: usize,
    desc: &'static StructDescriptor,
    addr: VromAddr,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    println!("{} {{", desc.name);

    for field in desc.fields {
        dump_field(scope, fs, segment_ctx, indent_level + 1, field, addr);
    }

    print!("{}}}", indent);
}

pub(super) fn dump_union<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    indent_level: usize,
    desc: &'static UnionDescriptor,
    addr: VromAddr,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    println!("{} {{", desc.name);

    let discriminant_addr = addr + desc.discriminant_offset;
    if let Some(discriminant) = desc
        .discriminant_desc
        .read_as_u32(scope, fs, discriminant_addr)
    {
        print!(
            "{}    (0x{:08x}) discriminant: {} = ",
            indent,
            addr.0,
            desc.discriminant_desc.name(),
        );
        dump(
            scope,
            fs,
            segment_ctx,
            indent_level + 1,
            desc.discriminant_desc,
            discriminant_addr,
        );
        println!(" (0x{:x})", discriminant);

        match desc.variants.get(discriminant as usize) {
            Some(Some(desc)) => {
                for field in desc.fields {
                    dump_field(scope, fs, segment_ctx, indent_level + 1, field, addr);
                }
            }
            _ => println!("{}    (unknown variant)", indent),
        }
    } else {
        println!("{}    (discriminant inaccessible)", indent);
    }

    print!("{}}}", indent);
}

fn dump_field<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    indent_level: usize,
    field: &'static FieldDescriptor,
    addr: VromAddr,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    match field.location {
        StructFieldLocation::Simple { offset } => {
            let addr = addr + offset;
            print!(
                "{}(0x{:08x}) {}: {} = ",
                indent,
                addr.0,
                field.name,
                field.desc.name(),
            );
            dump(scope, fs, segment_ctx, indent_level, field.desc, addr);
            println!();
        }
        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            let count_addr = addr + count_offset;
            let count = count_desc.read_as_u32(scope, fs, count_addr).ok();
            print!(
                "{}(0x{:08x}) {}_count: {} = ",
                indent,
                count_addr.0,
                field.name,
                count_desc.name(),
            );
            match count {
                Some(count) => println!("{}", count),
                None => println!("(inaccessible)"),
            }

            let ptr_addr = addr + ptr_offset;
            let ptr = fs
                .get_virtual_slice(scope, ptr_addr..ptr_addr + 4)
                .ok()
                .map(|mut data| SegmentAddr(data.read_u32::<BigEndian>().unwrap()));
            print!(
                "{}(0x{:08x}) {}_ptr: &{} = ",
                indent,
                ptr_addr.0,
                field.name,
                field.desc.name(),
            );
            match ptr {
                Some(addr) => println!("{:?}", addr),
                None => println!("(inaccessible)"),
            }

            if let (Some(count), Some(ptr)) = (count, ptr) {
                match segment_ctx.resolve_vrom(ptr) {
                    Ok(range) => {
                        let mut addr = range.start;
                        println!(
                            "{}{}: &[{}; {}] = &[",
                            indent,
                            field.name,
                            field.desc.name(),
                            count,
                        );

                        for _ in 0..count {
                            print!("{}    (0x{:08x}) ", indent, addr.0);
                            dump(scope, fs, segment_ctx, indent_level + 1, field.desc, addr);
                            println!();

                            addr += field.desc.size().expect("slice element has no size");
                        }

                        println!("{}]", indent)
                    }
                    Err(SegmentResolveError::Unmapped { .. }) => {
                        print!(
                            "{}{}: &[{}; {}] = (segment 0x{:x} not mapped)",
                            indent,
                            field.name,
                            field.desc.name(),
                            count,
                            ptr.segment().0,
                        );
                    }
                }
            } else {
                println!(
                    "{}(missing information to evaluate field `{}`)",
                    indent, field.name
                );
            }
        }
        StructFieldLocation::InlineDelimitedList { offset } => {
            let mut addr = addr + offset;
            println!(
                "{}(0x{:08x}) {}: [{}; N] = [",
                indent,
                addr.0,
                field.name,
                field.desc.name(),
            );

            loop {
                print!("{}    (0x{:08x}) ", indent, addr.0);
                dump(scope, fs, segment_ctx, indent_level + 1, field.desc, addr);
                println!();

                if (field
                    .desc
                    .is_end()
                    .expect("inline delimited list element has no is_end"))(
                    scope, fs, addr
                ) {
                    break;
                }

                addr += field
                    .desc
                    .size()
                    .expect("inline delimited list element has no size");
            }

            println!("{}]", indent)
        }
    }
}
