use deref_map::DerefMapExt;
use oot_explorer_core::fs::{LazyFileSystem, VromAddr};
use oot_explorer_core::reflect::instantiate::Instantiate;
use oot_explorer_core::reflect::primitive::PrimitiveType;
use oot_explorer_core::reflect::sized::ReflectSized;
use oot_explorer_core::reflect::struct_::StructFieldLocation;
use oot_explorer_core::reflect::type_::TypeDescriptor;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::segment::{Segment, SegmentAddr, SegmentCtx};
use oot_explorer_core::versions::oot_ntsc_10;
use scoped_owner::ScopedOwner;
use std::fmt::{Display, Write};
use std::ops::{Deref, Range};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Document, HtmlElement};

use crate::InnerContext;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Nesting {
    Even,
    Odd,
}

impl Nesting {
    pub fn next(self) -> Nesting {
        match self {
            Nesting::Even => Nesting::Odd,
            Nesting::Odd => Nesting::Even,
        }
    }
}

impl Default for Nesting {
    fn default() -> Self {
        Nesting::Even
    }
}

enum ExpandState {
    NotExpandable,
    Collapsed {
        indicator: HtmlElement,
        click_handler: Option<Closure<dyn Fn()>>,
    },
    Expanded {
        indicator: HtmlElement,
        click_handler: Option<Closure<dyn Fn()>>,
    },
}

impl ExpandState {
    fn mark_collapsed(&mut self) {
        // Temporarily replace self.
        let old_self = std::mem::replace(self, ExpandState::NotExpandable);

        match old_self {
            ExpandState::NotExpandable => {
                *self = old_self;
                panic!();
            }
            ExpandState::Collapsed { .. } => *self = old_self,
            ExpandState::Expanded {
                indicator,
                click_handler: indicator_click_handler,
            } => {
                indicator.class_list().remove_1("expanded").unwrap_throw();
                *self = ExpandState::Collapsed {
                    indicator,
                    click_handler: indicator_click_handler,
                };
            }
        }
    }

    fn mark_expanded(&mut self) {
        // Temporarily replace self.
        let old_self = std::mem::replace(self, ExpandState::NotExpandable);

        match old_self {
            ExpandState::NotExpandable => {
                *self = old_self;
                panic!();
            }
            ExpandState::Collapsed {
                indicator,
                click_handler: indicator_click_handler,
            } => {
                indicator.class_list().add_1("expanded").unwrap_throw();
                *self = ExpandState::Expanded {
                    indicator,
                    click_handler: indicator_click_handler,
                };
            }
            ExpandState::Expanded { .. } => *self = old_self,
        }
    }
}

#[derive(Clone)]
pub struct ItemView {
    inner: Arc<Mutex<InnerItemView>>,
}

impl ItemView {
    pub fn for_value<'scope>(
        document: &Document,
        ctx: Arc<Mutex<InnerContext>>,
        nesting: Nesting,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        segment_ctx: &SegmentCtx<'scope>,
        addr: VromAddr,
        desc: TypeDescriptor,
    ) -> ItemView {
        ItemView::for_field(
            document,
            ctx,
            nesting,
            scope,
            fs,
            segment_ctx,
            addr,
            None,
            StructFieldLocation::Simple { offset: 0 },
            desc,
        )
    }

    fn for_field<'scope>(
        document: &Document,
        ctx: Arc<Mutex<InnerContext>>,
        nesting: Nesting,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        segment_ctx: &SegmentCtx<'scope>,
        base_addr: VromAddr,
        field_name: Option<String>,
        location: StructFieldLocation,
        desc: TypeDescriptor,
    ) -> ItemView {
        // Format the fully decorated type name.
        let mut type_text = desc.name().to_string();
        match location {
            StructFieldLocation::Simple { .. } => (),
            StructFieldLocation::Slice { .. } => {
                write!(&mut type_text, "[]*").unwrap_throw();
            }
            StructFieldLocation::InlineDelimitedList { .. } => {
                write!(&mut type_text, "[..]").unwrap_throw();
            }
        }

        // Determine the field's address.
        let field_addr = get_field_vrom_range(base_addr, &location, desc).start;

        // Generate the skeleton of the item element tree.
        html_template!(document, let element = div[class="tree-item"] {
            let header = div[class="tree-item-header"] {
                span[class="tree-item-addr"] { text(&format!("{:08x}", field_addr.0)) }
                span[] { text("  ") }
                let type_element = span[class="tree-item-type"] { text(&type_text) }
            }
            let contents = div[class="tree-item-contents"] {}
        });
        element
            .class_list()
            .add_1(match nesting {
                Nesting::Even => "even",
                Nesting::Odd => "odd",
            })
            .unwrap_throw();

        // Add a span element for the item's field name.
        if let Some(ref name) = field_name {
            let name_element = html_template!(document, return
                span[class="tree-item-field"] { text(&format!("{}: ", name)) }
            );
            header
                .insert_before(&name_element, Some(&type_element))
                .unwrap_throw();
        }

        // Add a span element for the item's value.
        if let Some(value) = field_value_string(scope, fs, segment_ctx, base_addr, &location, desc)
        {
            html_template!(document, in header: span[class="tree-item-value"] {
                text(&format!(" = {}", value))
            });
        }

        // Add a div element for the expansion indicator.
        let expand_state = match field_is_expandable(scope, fs, base_addr, location.clone(), desc) {
            true => {
                html_template!(document, in element:
                    let indicator = div[class="tree-item-indicator"] { }
                );
                ExpandState::Collapsed {
                    indicator,
                    click_handler: None,
                }
            }
            false => ExpandState::NotExpandable,
        };

        // Create the inner Arc<Mutex> for event handlers to reference.
        let inner = Arc::new(Mutex::new(InnerItemView {
            ctx,
            base_addr,
            field_name,
            location,
            desc,

            element,
            expand_state,
            nesting,
            contents,
            fields: vec![],
            highlight_listener: None,
            mouseenter_handler: None,
            mouseleave_handler: None,
        }));
        let mut inner_mut = inner.lock().unwrap_throw();

        // Attach a click event handler to the indicator element.
        if let ExpandState::Collapsed {
            ref indicator,
            click_handler,
        } = &mut inner_mut.expand_state
        {
            *click_handler = Some(Closure::wrap(Box::new({
                let weak_inner = Arc::downgrade(&inner);
                move || {
                    if let Some(inner) = weak_inner.upgrade() {
                        inner.lock().unwrap_throw().handle_indicator_click();
                    }
                }
            }) as Box<dyn Fn()>));

            indicator
                .add_event_listener_with_callback(
                    "click",
                    click_handler
                        .as_ref()
                        .unwrap_throw()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap_throw();
        }

        // Attach mousein and mouseout event handlers to the header element.
        inner_mut.mouseenter_handler = Some(Closure::wrap(Box::new({
            let weak_inner = Arc::downgrade(&inner);
            move || {
                if let Some(inner) = weak_inner.upgrade() {
                    let inner_ref = inner.lock().unwrap_throw();
                    if let Some(listener) = inner_ref.highlight_listener.as_ref() {
                        listener(Some(get_field_vrom_range(
                            inner_ref.base_addr,
                            &inner_ref.location,
                            inner_ref.desc,
                        )));
                    }
                }
            }
        }) as Box<dyn Fn()>));
        inner_mut.mouseleave_handler = Some(Closure::wrap(Box::new({
            let weak_inner = Arc::downgrade(&inner);
            move || {
                if let Some(inner) = weak_inner.upgrade() {
                    let inner_ref = inner.lock().unwrap_throw();
                    if let Some(listener) = inner_ref.highlight_listener.as_ref() {
                        listener(None);
                    }
                }
            }
        }) as Box<dyn Fn()>));
        header
            .add_event_listener_with_callback(
                "mouseenter",
                inner_mut
                    .mouseenter_handler
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap_throw();
        header
            .add_event_listener_with_callback(
                "mouseleave",
                inner_mut
                    .mouseleave_handler
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap_throw();

        drop(inner_mut);
        ItemView { inner }
    }

    pub fn element(&self) -> impl Deref<Target = HtmlElement> + '_ {
        self.inner
            .lock()
            .unwrap_throw()
            .map_ref(|inner| &inner.element)
    }

    pub fn expand<'scope>(
        &mut self,
        document: &Document,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        segment_ctx: &SegmentCtx<'scope>,
    ) {
        self.inner
            .lock()
            .unwrap_throw()
            .expand(document, scope, fs, segment_ctx);
    }

    pub fn set_highlight_listener(&self, listener: Option<Arc<dyn Fn(Option<Range<VromAddr>>)>>) {
        let mut inner_mut = self.inner.lock().unwrap_throw();
        for field in &mut inner_mut.fields {
            field.set_highlight_listener(listener.clone());
        }
        inner_mut.highlight_listener = listener;
    }
}

struct InnerItemView {
    ctx: Arc<Mutex<InnerContext>>,
    base_addr: VromAddr,
    field_name: Option<String>,
    location: StructFieldLocation,
    desc: TypeDescriptor,

    element: HtmlElement,
    expand_state: ExpandState,
    nesting: Nesting,
    contents: HtmlElement,
    fields: Vec<ItemView>,
    highlight_listener: Option<Arc<dyn Fn(Option<Range<VromAddr>>)>>,
    mouseenter_handler: Option<Closure<dyn Fn()>>,
    mouseleave_handler: Option<Closure<dyn Fn()>>,
}

impl InnerItemView {
    pub fn collapse(&mut self) {
        match self.expand_state {
            ExpandState::Expanded { .. } => (),
            _ => return,
        }

        self.fields.clear();

        self.expand_state.mark_collapsed();
    }

    pub fn expand<'scope>(
        &mut self,
        document: &Document,
        scope: &'scope ScopedOwner,
        fs: &mut LazyFileSystem<'scope>,
        segment_ctx: &SegmentCtx<'scope>,
    ) {
        match self.expand_state {
            ExpandState::Collapsed { .. } => {}
            _ => return,
        }

        match self.location {
            StructFieldLocation::Simple { offset } => {
                // This instance represents a simple field. Dig in and add any of its fields.
                add_items_for_fields(
                    document,
                    &self.ctx,
                    self.nesting,
                    &self.contents,
                    &mut self.fields,
                    &self.highlight_listener,
                    scope,
                    fs,
                    segment_ctx,
                    self.desc,
                    self.base_addr + offset,
                );
            }
            StructFieldLocation::Slice {
                count_offset,
                count_desc,
                ptr_offset,
            } => {
                // This instance represents a slice field.

                // Retrieve the count.
                let count = count_desc
                    .read_as_u32(scope, fs, self.base_addr + count_offset)
                    .expect("not ready to make this robust yet");

                // Retrieve the initial pointer.
                let ptr_addr = self.base_addr + ptr_offset;
                let segment_ptr = <SegmentAddr as Instantiate>::new(
                    fs.get_virtual_slice(scope, ptr_addr..ptr_addr + 4)
                        .expect("not ready to make this robust yet"),
                );
                let mut vrom_ptr = segment_ctx
                    .resolve_vrom(segment_ptr)
                    .expect("not ready to make this robust yet")
                    .start;

                // Add a field for each value in the slice.
                for index in 0..count {
                    let field_view = ItemView::for_field(
                        document,
                        Arc::clone(&self.ctx),
                        self.nesting.next(),
                        scope,
                        fs,
                        segment_ctx,
                        vrom_ptr,
                        Some(format!("{}", index)),
                        StructFieldLocation::Simple { offset: 0 },
                        self.desc,
                    );
                    self.contents
                        .append_child(&*field_view.element())
                        .unwrap_throw();
                    field_view.set_highlight_listener(self.highlight_listener.clone());
                    self.fields.push(field_view);

                    vrom_ptr += match self.desc.size() {
                        Some(size) => size,
                        None => panic!(
                            "slice element {} has no size, referenced from field {:?}",
                            self.desc.name(),
                            self.field_name,
                        ),
                    };
                }
            }
            StructFieldLocation::InlineDelimitedList { offset } => {
                // This instance represents an inline delimited list field.

                // Retrieve the is_end function.
                let is_end = match self.desc.is_end() {
                    Some(is_end) => is_end,
                    None => panic!(
                        "delimited list element {} has no is_end, referenced from field {:?}",
                        self.desc.name(),
                        self.field_name,
                    ),
                };

                // Add a field for each value in the list.
                let mut ptr = self.base_addr + offset;
                for index in 0.. {
                    let field_view = ItemView::for_field(
                        document,
                        Arc::clone(&self.ctx),
                        self.nesting.next(),
                        scope,
                        fs,
                        segment_ctx,
                        ptr,
                        Some(format!("{}", index)),
                        StructFieldLocation::Simple { offset: 0 },
                        self.desc,
                    );
                    self.contents
                        .append_child(&*field_view.element())
                        .unwrap_throw();
                    field_view.set_highlight_listener(self.highlight_listener.clone());
                    self.fields.push(field_view);

                    if is_end(scope, fs, ptr) {
                        break;
                    }

                    ptr += match self.desc.size() {
                        Some(size) => size,
                        None => panic!(
                            "delimited list element {} has no size, referenced from field {:?}",
                            self.desc.name(),
                            self.field_name,
                        ),
                    };
                }
            }
        }

        self.expand_state.mark_expanded();
    }

    fn handle_indicator_click(&mut self) {
        match self.expand_state {
            ExpandState::NotExpandable => unreachable!(),
            ExpandState::Collapsed { .. } => {
                // TODO: Need a way to describe and persist the segment mappings. Also, drilling
                // down into some fields (e.g., room entries) needs to be able to set segments
                // for descending items.

                let ctx = Arc::clone(&self.ctx);
                let ctx_ref = ctx.lock().unwrap_throw();

                let document = web_sys::window().unwrap_throw().document().unwrap_throw();
                ScopedOwner::with_scope(|scope| {
                    let mut fs = LazyFileSystem::new(
                        Rom::new(&*ctx_ref.rom_data),
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

                    self.expand(&document, scope, &mut fs, &segment_ctx);
                });
            }
            ExpandState::Expanded { .. } => self.collapse(),
        }
    }
}

impl Drop for InnerItemView {
    fn drop(&mut self) {
        if let Some(parent) = self.element.parent_element() {
            parent.remove_child(&self.element).unwrap_throw();
        }
    }
}

fn field_is_expandable<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    base_addr: VromAddr,
    location: StructFieldLocation,
    desc: TypeDescriptor,
) -> bool {
    match location {
        StructFieldLocation::Simple { .. } => simple_type_is_expandable(desc),
        StructFieldLocation::Slice {
            count_offset,
            count_desc,
            ptr_offset,
        } => {
            // Should be expandable if the count and pointer are accessible. The expanded items will
            // indicate whether the pointer can be segment-resolved.
            if let Err(_) = count_desc.read_as_u32(scope, fs, base_addr + count_offset) {
                return false;
            }
            let ptr_addr = base_addr + ptr_offset;
            if let Err(_) = fs.get_virtual_slice(scope, ptr_addr..ptr_addr + 4) {
                return false;
            }
            true
        }
        StructFieldLocation::InlineDelimitedList { .. } => true,
    }
}

fn simple_type_is_expandable(desc: TypeDescriptor) -> bool {
    match desc {
        TypeDescriptor::Struct(_) | TypeDescriptor::Union(_) => true,
        TypeDescriptor::Enum(_) | TypeDescriptor::Bitfield(_) | TypeDescriptor::Primitive(_) => {
            false
        }
        TypeDescriptor::Pointer(pointer_desc) => simple_type_is_expandable(pointer_desc.target),
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

fn add_items_for_fields<'scope>(
    document: &Document,
    ctx: &Arc<Mutex<InnerContext>>,
    nesting: Nesting,
    contents: &HtmlElement,
    fields: &mut Vec<ItemView>,
    highlight_listener: &Option<Arc<dyn Fn(Option<Range<VromAddr>>)>>,
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    segment_ctx: &SegmentCtx<'scope>,
    desc: TypeDescriptor,
    addr: VromAddr,
) {
    match desc {
        TypeDescriptor::Struct(desc) => {
            for field in desc.fields {
                // Add an item for the field.
                let field_view = ItemView::for_field(
                    document,
                    Arc::clone(ctx),
                    nesting.next(),
                    scope,
                    fs,
                    segment_ctx,
                    addr,
                    Some(field.name.to_string()),
                    field.location.clone(),
                    field.desc,
                );
                contents.append_child(&*field_view.element()).unwrap_throw();
                field_view.set_highlight_listener(highlight_listener.clone());
                fields.push(field_view);
            }
        }

        TypeDescriptor::Union(union_desc) => {
            // Add an item for the discriminant.
            let discriminant_view = ItemView::for_field(
                document,
                Arc::clone(ctx),
                nesting.next(),
                scope,
                fs,
                segment_ctx,
                addr,
                Some("discriminant".to_string()),
                StructFieldLocation::Simple {
                    offset: union_desc.discriminant_offset,
                },
                union_desc.discriminant_desc,
            );
            contents
                .append_child(&*discriminant_view.element())
                .unwrap_throw();
            discriminant_view.set_highlight_listener(highlight_listener.clone());
            fields.push(discriminant_view);

            // If the discriminant is accessible and known, recurse to add items for each field
            // in the variant.
            match union_desc.discriminant_desc.read_as_u32(
                scope,
                fs,
                addr + union_desc.discriminant_offset,
            ) {
                Some(discriminant) => {
                    match union_desc
                        .variants
                        .binary_search_by_key(&discriminant, |&(x, _)| x)
                    {
                        Ok(index) => {
                            let variant_desc = union_desc.variants[index].1;
                            add_items_for_fields(
                                document,
                                ctx,
                                nesting,
                                contents,
                                fields,
                                highlight_listener,
                                scope,
                                fs,
                                segment_ctx,
                                variant_desc,
                                addr,
                            );
                        }
                        Err(_) => {}
                    }
                }
                None => {
                    //
                }
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
            let item_view = ItemView::for_field(
                document,
                Arc::clone(ctx),
                nesting.next(),
                scope,
                fs,
                segment_ctx,
                vrom_ptr,
                None,
                StructFieldLocation::Simple { offset: 0 },
                pointer_desc.target,
            );
            contents.append_child(&*item_view.element()).unwrap_throw();
            item_view.set_highlight_listener(highlight_listener.clone());
            fields.push(item_view);
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
