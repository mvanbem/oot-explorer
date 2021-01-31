use oot_explorer_read::{FromVrom, ReadError};
use oot_explorer_reflect::{
    BitfieldDescriptor, EnumDescriptor, FieldDescriptor, PointerDescriptor, PrimitiveType,
    StructDescriptor, StructFieldLocation, TypeDescriptor, UnionDescriptor,
};
use oot_explorer_segment::{SegmentAddr, SegmentTable};
use oot_explorer_vrom::{Vrom, VromAddr};

pub fn dump(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: TypeDescriptor,
    addr: VromAddr,
    indent_level: usize,
) {
    match desc {
        TypeDescriptor::Struct(desc) => {
            dump_struct(vrom, segment_table, desc, addr, indent_level);
        }
        TypeDescriptor::Union(desc) => {
            dump_union(vrom, segment_table, desc, addr, indent_level);
        }
        TypeDescriptor::Enum(desc) => dump_enum(vrom, desc, addr),
        TypeDescriptor::Bitfield(desc) => dump_bitfield(vrom, desc, addr),
        TypeDescriptor::Primitive(desc) => dump_primitive(vrom, desc, addr),
        TypeDescriptor::Pointer(desc) => {
            dump_pointer(vrom, segment_table, desc, addr, indent_level)
        }
    }
}

fn dump_bitfield(vrom: Vrom<'_>, desc: &'static BitfieldDescriptor, addr: VromAddr) -> () {
    let value = match desc.underlying.read_as_u32(vrom, addr) {
        Ok(value) => value,
        Err(e) => {
            print!("{}", e);
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

fn dump_enum(vrom: Vrom<'_>, desc: &'static EnumDescriptor, addr: VromAddr) {
    match desc.underlying.read_as_u32(vrom, addr) {
        Ok(value) => match desc.values.binary_search_by_key(&value, |&(x, _)| x) {
            Ok(index) => print!("{}", desc.values[index].1),
            Err(_) => print!("(unknown value 0x{:x}", value),
        },
        Err(e) => print!("{}", e),
    }
}

fn dump_primitive(vrom: Vrom<'_>, desc: PrimitiveType, addr: VromAddr) -> () {
    let try_print = || {
        match desc {
            PrimitiveType::Bool => print!("{}", bool::from_vrom(vrom, addr)?),
            PrimitiveType::U8 => print!("{}", u8::from_vrom(vrom, addr)?),
            PrimitiveType::I8 => print!("{}", i8::from_vrom(vrom, addr)?),
            PrimitiveType::U16 => {
                print!("{}", u16::from_vrom(vrom, addr)?)
            }
            PrimitiveType::I16 => {
                print!("{}", i16::from_vrom(vrom, addr)?)
            }
            PrimitiveType::U32 => {
                print!("{}", u32::from_vrom(vrom, addr)?)
            }
            PrimitiveType::I32 => {
                print!("{}", i32::from_vrom(vrom, addr)?)
            }
            PrimitiveType::VromAddr => {
                print!("{:?}", VromAddr::from_vrom(vrom, addr)?)
            }
            PrimitiveType::SegmentAddr => {
                print!("{:?}", SegmentAddr::from_vrom(vrom, addr)?)
            }
        }
        Result::<(), ReadError>::Ok(())
    };

    if let Err(e) = try_print() {
        print!("{}", e);
    }
}

fn dump_pointer(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: &'static PointerDescriptor,
    addr: VromAddr,
    indent_level: usize,
) {
    let segment_addr = match SegmentAddr::from_vrom(vrom, addr) {
        Ok(segment_addr) => segment_addr,
        Err(e) => {
            print!("{}", e);
            return;
        }
    };

    let vrom_addr = match segment_table.resolve(segment_addr) {
        Ok(vrom_addr) => vrom_addr,
        Err(e) => {
            print!("{}", e);
            return;
        }
    };

    print!("&");
    dump(vrom, segment_table, desc.target, vrom_addr, indent_level)
}

fn dump_struct(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: &'static StructDescriptor,
    addr: VromAddr,
    indent_level: usize,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    println!("{} {{", desc.name);

    for field in desc.fields {
        dump_field(vrom, segment_table, field, addr, indent_level + 1);
    }

    print!("{}}}", indent);
}

fn dump_union(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: &'static UnionDescriptor,
    addr: VromAddr,
    indent_level: usize,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    println!("{} {{", desc.name);
    dump_union_body(vrom, segment_table, desc, addr, indent_level + 1);
    print!("{}}}", indent);
}

fn dump_union_body(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: &'static UnionDescriptor,
    addr: VromAddr,
    indent_level: usize,
) {
    let indent = std::iter::repeat(' ')
        .take(4 * indent_level)
        .collect::<String>();

    let discriminant_addr = addr + desc.discriminant_offset;
    match desc
        .discriminant_desc
        .read_as_u32(vrom, discriminant_addr)
        .expect("enum discriminant must be readable as u32")
    {
        Ok(discriminant) => {
            print!(
                "{}(0x{:08x}) discriminant: {} = ",
                indent,
                discriminant_addr.0,
                desc.discriminant_desc.name(),
            );
            dump(
                vrom,
                segment_table,
                desc.discriminant_desc,
                discriminant_addr,
                indent_level,
            );
            println!(" (0x{:x})", discriminant);

            match desc
                .variants
                .binary_search_by_key(&discriminant, |&(x, _)| x)
            {
                Ok(index) => match desc.variants[index].1 {
                    TypeDescriptor::Struct(desc) => {
                        for field in desc.fields {
                            dump_field(vrom, segment_table, field, addr, indent_level);
                        }
                    }
                    TypeDescriptor::Union(desc) => {
                        dump_union_body(vrom, segment_table, desc, addr, indent_level);
                    }
                    _ => unimplemented!(
                        "variant `{}` of union `{}` is not a struct or union",
                        desc.variants[index].1.name(),
                        desc.name,
                    ),
                },
                Err(_) => {
                    println!("{}(unknown variant)", indent);
                }
            }
        }
        Err(e) => {
            println!("{}{}", indent, e);
        }
    }
}

fn dump_field(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    field: &'static FieldDescriptor,
    addr: VromAddr,
    indent_level: usize,
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
            dump(vrom, segment_table, field.desc, addr, indent_level);
            println!();
        }
        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            let count_addr = addr + count_offset;
            print!(
                "{}(0x{:08x}) {}_count: {} = ",
                indent,
                count_addr.0,
                field.name,
                count_desc.name(),
            );
            let count = match count_desc.read_as_u32(vrom, count_addr) {
                Ok(count) => {
                    println!("{}", count);
                    Some(count)
                }
                Err(e) => {
                    println!("{}", e);
                    None
                }
            };

            let ptr_addr = addr + ptr_offset;
            print!(
                "{}(0x{:08x}) {}_ptr: &{} = ",
                indent,
                ptr_addr.0,
                field.name,
                field.desc.name(),
            );
            let segment_ptr = match SegmentAddr::from_vrom(vrom, ptr_addr) {
                Ok(segment_ptr) => {
                    println!("{:?}", segment_ptr);
                    Some(segment_ptr)
                }
                Err(e) => {
                    println!("{}", e);
                    None
                }
            };

            if let (Some(count), Some(segment_ptr)) = (count, segment_ptr) {
                match segment_table.resolve(segment_ptr) {
                    Ok(mut vrom_addr) => {
                        println!(
                            "{}{}: &[{}; {}] = &[",
                            indent,
                            field.name,
                            field.desc.name(),
                            count,
                        );

                        for _ in 0..count {
                            print!("{}    (0x{:08x}) ", indent, vrom_addr.0);
                            dump(vrom, segment_table, field.desc, vrom_addr, indent_level + 1);
                            println!();

                            vrom_addr += match field.desc.size() {
                                Some(size) => size,
                                None => panic!(
                                    "slice element {} has no size, referenced from field {}",
                                    field.desc.name(),
                                    field.name,
                                ),
                            };
                        }

                        println!("{}]", indent)
                    }
                    Err(e) => {
                        print!(
                            "{}{}: &[{}; {}] = {}",
                            indent,
                            field.name,
                            field.desc.name(),
                            count,
                            e,
                        );
                    }
                }
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
                dump(vrom, segment_table, field.desc, addr, indent_level + 1);
                println!();

                if (field
                    .desc
                    .is_end()
                    .expect("inline delimited list element has no is_end"))(
                    vrom, addr
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
