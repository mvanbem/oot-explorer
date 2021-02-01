use oot_explorer_read::{FromVrom, Layout};
use oot_explorer_reflect::{PrimitiveType, StructFieldLocation, TypeDescriptor};
use oot_explorer_segment::{SegmentAddr, SegmentTable};
use oot_explorer_vrom::{Vrom, VromAddr};
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::ops::Range;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::reflect_root::ReflectRoot;
use crate::Context;

#[wasm_bindgen]
pub struct ReflectResult {
    info: ReflectItemInfo,
    fields: Vec<ReflectFieldInfo>,
}

#[wasm_bindgen]
impl ReflectResult {
    #[wasm_bindgen(getter)]
    pub fn info(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.info).unwrap_throw()
    }

    #[wasm_bindgen(getter = fieldsCount)]
    pub fn fields_count(&self) -> usize {
        self.fields.len()
    }

    #[wasm_bindgen(js_name = getField)]
    pub fn get_field(&self, index: usize) -> ReflectFieldInfo {
        self.fields[index].clone()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReflectItemInfo {
    base_addr: u32,
    field_name: Option<String>,
    type_string: String,
    value_string: Option<String>,
    vrom_start: u32,
    vrom_end: u32,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ReflectFieldInfo {
    name: Option<String>,
    base_addr: VromAddr,
    location: StructFieldLocation,
    desc: TypeDescriptor,
}

#[wasm_bindgen]
impl ReflectFieldInfo {
    pub fn reflect(&self, ctx: &Context, root: &ReflectRoot) -> ReflectResult {
        let ctx_ref = ctx.inner.lock().unwrap_throw();
        let vrom = ctx_ref.vrom.as_ref().unwrap_throw().borrow();

        reflect_field(
            vrom,
            &root.segment_table,
            self.base_addr,
            self.name.clone(),
            &self.location,
            self.desc,
        )
    }
}

pub fn reflect_field(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    base_addr: VromAddr,
    field_name: Option<String>,
    location: &StructFieldLocation,
    desc: TypeDescriptor,
) -> ReflectResult {
    // Format the fully decorated type name.
    let type_string = desc.name().to_string()
        + match location {
            StructFieldLocation::Simple { .. } => "",
            StructFieldLocation::Slice { .. } => "[]*",
            StructFieldLocation::InlineDelimitedList { .. } => "[..]",
        };

    let vrom_range = get_field_vrom_range(base_addr, &location, desc);
    let value_string = field_value_string(vrom, segment_table, base_addr, &location, desc);
    let contents = contents(vrom, segment_table, base_addr, &location, desc);

    ReflectResult {
        info: ReflectItemInfo {
            base_addr: base_addr.0,
            field_name,
            type_string,
            value_string,
            vrom_start: vrom_range.start.0,
            vrom_end: vrom_range.end.0,
        },
        fields: contents,
    }
}

fn contents(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    base_addr: VromAddr,
    location: &StructFieldLocation,
    desc: TypeDescriptor,
) -> Vec<ReflectFieldInfo> {
    match location {
        StructFieldLocation::Simple { offset } => {
            // This instance represents a simple field. Dig in and add any of its fields.
            let mut field_infos = vec![];
            add_field_infos_for_fields(
                vrom,
                segment_table,
                desc,
                base_addr + *offset,
                &mut field_infos,
            );
            field_infos
        }
        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            // This instance represents a slice field.

            // Retrieve the count.
            let count = count_desc
                .read_as_u32(vrom, base_addr + *count_offset)
                .expect("not ready to make this robust yet");

            // Retrieve the initial pointer.
            let ptr_addr = base_addr + *ptr_offset;
            let segment_ptr =
                SegmentAddr::from_vrom(vrom, ptr_addr).expect("not ready to make this robust yet");
            if segment_ptr.is_null() {
                return vec![];
            }

            // Resolve the segment address. If it's unmapped, we have no contents. The slice field's
            // one-line value should display the error message.
            let mut vrom_ptr = match segment_table.resolve(segment_ptr) {
                Ok(vrom_ptr) => vrom_ptr,
                Err(_) => return vec![],
            };

            // Add a field for each value in the slice.
            let mut field_infos = vec![];
            for index in 0..count {
                field_infos.push(ReflectFieldInfo {
                    name: Some(format!("{}", index)),
                    base_addr: vrom_ptr,
                    location: StructFieldLocation::Simple { offset: 0 },
                    desc,
                });

                vrom_ptr += match desc.size() {
                    Some(size) => size,
                    None => panic!("slice element {} has no size", desc.name()),
                };
            }
            field_infos
        }
        StructFieldLocation::InlineDelimitedList { offset } => {
            // This instance represents an inline delimited list field.

            // Retrieve the is_end function.
            let is_end = match desc.is_end() {
                Some(is_end) => is_end,
                None => panic!("delimited list element {} has no is_end", desc.name()),
            };

            // Add a field for each value in the list.
            let mut field_infos = vec![];
            let mut ptr = base_addr + *offset;
            for index in 0.. {
                field_infos.push(ReflectFieldInfo {
                    name: Some(format!("{}", index)),
                    base_addr: ptr,
                    location: StructFieldLocation::Simple { offset: 0 },
                    desc,
                });

                if is_end(vrom, ptr) {
                    break;
                }

                ptr += match desc.size() {
                    Some(size) => size,
                    None => panic!("delimited list element {} has no size", desc.name()),
                };
            }
            field_infos
        }
    }
}

fn get_field_vrom_range(
    base_addr: VromAddr,
    location: &StructFieldLocation,
    desc: TypeDescriptor,
) -> Range<VromAddr> {
    let (offset, known_size) = match location {
        StructFieldLocation::Simple { offset } => (*offset, None),
        StructFieldLocation::Slice { ptr_offset, .. } => (*ptr_offset, Some(SegmentAddr::SIZE)),
        StructFieldLocation::InlineDelimitedList { offset } => (*offset, None),
    };

    let addr = base_addr + offset;
    match known_size.or_else(|| desc.size()) {
        Some(size) => addr..addr + size,
        None => addr..addr + 1,
    }
}

fn add_field_infos_for_fields(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    desc: TypeDescriptor,
    addr: VromAddr,
    field_infos: &mut Vec<ReflectFieldInfo>,
) {
    match desc {
        TypeDescriptor::Struct(desc) => {
            field_infos.extend(desc.fields.iter().map(|field| ReflectFieldInfo {
                name: Some(field.name.to_string()),
                base_addr: addr,
                location: field.location.clone(),
                desc: field.desc,
            }))
        }

        TypeDescriptor::Union(union_desc) => {
            // Add an item for the discriminant.
            field_infos.push(ReflectFieldInfo {
                name: Some("discriminant".to_string()),
                base_addr: addr,
                location: StructFieldLocation::Simple {
                    offset: union_desc.discriminant_offset,
                },
                desc: union_desc.discriminant_desc,
            });

            // If the discriminant is accessible and known, recurse to add items for each field in
            // the variant.
            let discriminant = match union_desc
                .discriminant_desc
                .read_as_u32(vrom, addr + union_desc.discriminant_offset)
                .expect("union discriminants must be readable as u32")
            {
                Ok(discriminant) => discriminant,
                Err(_) => return,
            };
            if let Ok(index) = union_desc
                .variants
                .binary_search_by_key(&discriminant, |&(x, _)| x)
            {
                let variant_desc = union_desc.variants[index].1;
                add_field_infos_for_fields(vrom, segment_table, variant_desc, addr, field_infos);
            }
        }

        TypeDescriptor::Pointer(pointer_desc) => {
            // TODO: Add pseudo-items for failure to dereference a pointer.

            let segment_ptr = match SegmentAddr::from_vrom(vrom, addr) {
                Ok(segment_ptr) => segment_ptr,
                Err(_) => return,
            };
            let vrom_ptr = match segment_table.resolve(segment_ptr) {
                Ok(vrom_ptr) => vrom_ptr,
                Err(_) => return,
            };

            // Add an item for the pointed-to value.
            field_infos.push(ReflectFieldInfo {
                name: None,
                base_addr: vrom_ptr,
                location: StructFieldLocation::Simple { offset: 0 },
                desc: pointer_desc.target,
            });
        }

        // These types don't have fields.
        TypeDescriptor::Enum(_) | TypeDescriptor::Bitfield(_) | TypeDescriptor::Primitive(_) => {}
    }
}

fn field_value_string(
    vrom: Vrom<'_>,
    _segment_table: &SegmentTable,
    base_addr: VromAddr,
    location: &StructFieldLocation,
    desc: TypeDescriptor,
) -> Option<String> {
    let fallible_result = (|| match location {
        StructFieldLocation::Simple { offset } => {
            let field_addr = base_addr + *offset;
            match desc {
                TypeDescriptor::Enum(enum_desc) => {
                    let value = enum_desc
                        .read_as_u32(vrom, field_addr)
                        .map_err(|_| format!("(inaccessible)"))?;
                    let index = enum_desc
                        .values
                        .binary_search_by_key(&value, |&(x, _)| x)
                        .map_err(|_| format!("(unknown value 0x{:x})", value))?;
                    Ok(Some(format!(
                        "{} (0x{:x})",
                        enum_desc.values[index].1, value
                    )))
                }

                TypeDescriptor::Bitfield(_) => Ok(Some(format!("(bitfields not implemented)"))),

                TypeDescriptor::Primitive(primitive) => match primitive {
                    PrimitiveType::Bool => fetch_and_display::<bool>(vrom, field_addr),
                    PrimitiveType::U8 => fetch_and_display::<u8>(vrom, field_addr),
                    PrimitiveType::I8 => fetch_and_display::<i8>(vrom, field_addr),
                    PrimitiveType::U16 => fetch_and_display::<u16>(vrom, field_addr),
                    PrimitiveType::I16 => fetch_and_display::<i16>(vrom, field_addr),
                    PrimitiveType::U32 => fetch_and_display::<u32>(vrom, field_addr),
                    PrimitiveType::I32 => fetch_and_display::<i32>(vrom, field_addr),
                    PrimitiveType::VromAddr => fetch_and_debug::<VromAddr>(vrom, field_addr),
                    PrimitiveType::SegmentAddr => fetch_and_debug::<SegmentAddr>(vrom, field_addr),
                },

                TypeDescriptor::Pointer(pointer_desc) => {
                    match SegmentAddr::from_vrom(vrom, field_addr) {
                        Ok(segment_ptr) => {
                            Ok(Some(format!("({}) {:?}", pointer_desc.name, segment_ptr)))
                        }
                        Err(_) => Ok(Some(format!("(inaccessible)"))),
                    }
                }

                TypeDescriptor::Union(union_desc) => {
                    let discriminant_value = field_value_string(
                        vrom,
                        _segment_table,
                        base_addr + union_desc.discriminant_offset,
                        &StructFieldLocation::Simple { offset: 0 },
                        union_desc.discriminant_desc,
                    )
                    .ok_or_else(|| "(inaccessible)".to_string())?;

                    Ok(Some(format!("{{ {}, .. }}", discriminant_value)))
                }

                // Structs do not have a one-line value string.
                TypeDescriptor::Struct(_) => Ok(None),
            }
        }

        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            let count = match count_desc.read_as_u32(vrom, base_addr + *count_offset) {
                Ok(count) => format!("{}", count),
                Err(_) => format!("(inaccessible)"),
            };
            let ptr_addr = base_addr + *ptr_offset;
            let ptr = match SegmentAddr::from_vrom(vrom, ptr_addr) {
                Ok(vrom_addr) => format!("{:?}", vrom_addr),
                Err(_) => format!("(inaccessible)"),
            };
            Ok(Some(format!("({}[{}]*) {}", desc.name(), count, ptr)))
        }
        StructFieldLocation::InlineDelimitedList { .. } => {
            // TODO
            Ok(None)
        }
    })();
    match fallible_result {
        Ok(result) => result,
        Err(message) => Some(message),
    }
}

fn fetch_and_display<T>(vrom: Vrom<'_>, addr: VromAddr) -> Result<Option<String>, String>
where
    T: Display + FromVrom + Layout,
{
    match T::from_vrom(vrom, addr) {
        Ok(value) => Ok(Some(format!("{}", value))),
        Err(_) => Err(format!("(inaccessible)")),
    }
}

fn fetch_and_debug<T>(vrom: Vrom<'_>, addr: VromAddr) -> Result<Option<String>, String>
where
    T: Debug + FromVrom + Layout,
{
    match T::from_vrom(vrom, addr) {
        Ok(value) => Ok(Some(format!("{:?}", value))),
        Err(_) => Err(format!("(inaccessible)")),
    }
}
