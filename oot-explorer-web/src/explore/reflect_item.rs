use oot_explorer_core::fs::{LazyFileSystem, VromAddr};
use oot_explorer_core::reflect::instantiate::Instantiate;
use oot_explorer_core::reflect::primitive::PrimitiveType;
use oot_explorer_core::reflect::sized::ReflectSized;
use oot_explorer_core::reflect::struct_::StructFieldLocation;
use oot_explorer_core::reflect::type_::TypeDescriptor;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::scene::SCENE_DESC;
use oot_explorer_core::segment::{Segment, SegmentAddr, SegmentCtx};
use oot_explorer_core::versions::oot_ntsc_10;
use scoped_owner::ScopedOwner;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Range;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

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
    base_addr: VromAddr,
    field_name: Option<String>,
    type_string: String,
    value_string: Option<String>,
    vrom_start: VromAddr,
    vrom_end: VromAddr,
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
    pub fn reflect_within_deku_tree_scene(&self, ctx: &Context) -> ReflectResult {
        let ctx_ref = ctx.inner.lock().unwrap_throw();
        ScopedOwner::with_scope(|scope| {
            let mut fs = LazyFileSystem::new(
                Rom::new(&ctx_ref.rom_data),
                oot_ntsc_10::FILE_TABLE_ROM_ADDR,
            );
            let scene_table = oot_ntsc_10::get_scene_table(scope, &mut fs);
            let scene = scene_table
                .iter()
                .next()
                .unwrap_throw()
                .scene(scope, &mut fs);
            let segment_ctx = {
                let mut segment_ctx = SegmentCtx::new();
                segment_ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
                segment_ctx
            };

            reflect_field(
                scope,
                &mut fs,
                &segment_ctx,
                self.base_addr,
                self.name.clone(),
                &self.location,
                self.desc,
            )
        })
    }
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn reflect_inside_the_deku_tree_scene(ctx: &Context) -> ReflectResult {
    // TODO: This is very hard-coded. Don't do this.

    let ctx_ref = ctx.inner.lock().unwrap_throw();
    ScopedOwner::with_scope(|scope| {
        let mut fs = LazyFileSystem::new(
            Rom::new(&ctx_ref.rom_data),
            oot_ntsc_10::FILE_TABLE_ROM_ADDR,
        );
        let scene_table = oot_ntsc_10::get_scene_table(scope, &mut fs);
        let scene = scene_table
            .iter()
            .next()
            .unwrap_throw()
            .scene(scope, &mut fs);
        let segment_ctx = {
            let mut segment_ctx = SegmentCtx::new();
            segment_ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
            segment_ctx
        };

        reflect_field(
            scope,
            &mut fs,
            &segment_ctx,
            scene.addr(),
            None,
            &StructFieldLocation::Simple { offset: 0 },
            SCENE_DESC,
        )
    })
}

fn reflect_field<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
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
    let value_string = field_value_string(scope, fs, segment_ctx, base_addr, &location, desc);
    let contents = contents(scope, fs, segment_ctx, base_addr, &location, desc);

    ReflectResult {
        info: ReflectItemInfo {
            base_addr,
            field_name,
            type_string,
            value_string,
            vrom_start: vrom_range.start,
            vrom_end: vrom_range.end,
        },
        fields: contents,
    }
}

pub fn contents<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    base_addr: VromAddr,
    location: &StructFieldLocation,
    desc: TypeDescriptor,
) -> Vec<ReflectFieldInfo> {
    match location {
        StructFieldLocation::Simple { offset } => {
            // This instance represents a simple field. Dig in and add any of its fields.
            let mut field_infos = vec![];
            add_field_infos_for_fields(
                scope,
                fs,
                segment_ctx,
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
                .read_as_u32(scope, fs, base_addr + *count_offset)
                .expect("not ready to make this robust yet");

            // Retrieve the initial pointer.
            let ptr_addr = base_addr + *ptr_offset;
            let segment_ptr = <SegmentAddr as Instantiate>::new(
                fs.get_virtual_slice(scope, ptr_addr..ptr_addr + 4)
                    .expect("not ready to make this robust yet"),
            );
            let mut vrom_ptr = segment_ctx
                .resolve_vrom(segment_ptr)
                .expect("not ready to make this robust yet")
                .start;

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

                if is_end(scope, fs, ptr) {
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
        StructFieldLocation::Slice { ptr_offset, .. } => {
            (*ptr_offset, Some(SegmentAddr::SIZE as u32))
        }
        StructFieldLocation::InlineDelimitedList { offset } => (*offset, None),
    };

    let addr = base_addr + offset;
    match known_size.or_else(|| desc.size()) {
        Some(size) => addr..addr + size,
        None => addr..addr + 1,
    }
}

fn add_field_infos_for_fields<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
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
            let discriminant = match union_desc.discriminant_desc.read_as_u32(
                scope,
                fs,
                addr + union_desc.discriminant_offset,
            ) {
                Some(discriminant) => discriminant,
                None => return,
            };
            if let Ok(index) = union_desc
                .variants
                .binary_search_by_key(&discriminant, |&(x, _)| x)
            {
                let variant_desc = union_desc.variants[index].1;
                add_field_infos_for_fields(scope, fs, segment_ctx, variant_desc, addr, field_infos);
            }
        }

        TypeDescriptor::Pointer(pointer_desc) => {
            // TODO: Add pseudo-items for failure to dereference a pointer.

            let segment_ptr = match fs.get_virtual_slice(scope, addr..addr + 4) {
                Ok(data) => <SegmentAddr as Instantiate>::new(data),
                Err(_) => return,
            };
            let vrom_ptr = match segment_ctx.resolve_vrom(segment_ptr) {
                Ok(range) => range.start,
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

fn field_value_string<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    _segment_ctx: &SegmentCtx<'scope>,
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
                        .read_as_u32(scope, fs, field_addr)
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
                    PrimitiveType::Bool => fetch_and_display::<bool>(scope, fs, field_addr),
                    PrimitiveType::U8 => fetch_and_display::<u8>(scope, fs, field_addr),
                    PrimitiveType::I8 => fetch_and_display::<i8>(scope, fs, field_addr),
                    PrimitiveType::U16 => fetch_and_display::<u16>(scope, fs, field_addr),
                    PrimitiveType::I16 => fetch_and_display::<i16>(scope, fs, field_addr),
                    PrimitiveType::U32 => fetch_and_display::<u32>(scope, fs, field_addr),
                    PrimitiveType::I32 => fetch_and_display::<i32>(scope, fs, field_addr),
                    PrimitiveType::VromAddr => Ok(None),
                    PrimitiveType::SegmentAddr => Ok(None),
                },

                TypeDescriptor::Pointer(pointer_desc) => {
                    match fs.get_virtual_slice(scope, field_addr..field_addr + 4) {
                        Ok(data) => {
                            let segment_ptr = <SegmentAddr as Instantiate>::new(data);
                            Ok(Some(format!("({}) {:?}", pointer_desc.name, segment_ptr)))
                        }
                        Err(_) => Ok(Some(format!("(inaccessible)",))),
                    }
                }

                // Structs and unions do not have a one-line value string.
                TypeDescriptor::Struct(_) | TypeDescriptor::Union(_) => Ok(None),
            }
        }

        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            let count = match count_desc.read_as_u32(scope, fs, base_addr + *count_offset) {
                Ok(count) => format!("{}", count),
                Err(_) => format!("(inaccessible)"),
            };
            let ptr_addr = base_addr + *ptr_offset;
            let ptr = match fs.get_virtual_slice(scope, ptr_addr..ptr_addr + 4) {
                Ok(data) => format!("{:?}", <SegmentAddr as Instantiate>::new(data)),
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

fn fetch_and_display<'scope, T>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    addr: VromAddr,
) -> Result<Option<String>, String>
where
    T: Display + Instantiate<'scope> + ReflectSized,
{
    let data = fs
        .get_virtual_slice(scope, addr..addr + <T as ReflectSized>::SIZE as u32)
        .map_err(|_| format!("(inaccessible)"))?;
    Ok(Some(format!("{}", <T as Instantiate>::new(data))))
}
