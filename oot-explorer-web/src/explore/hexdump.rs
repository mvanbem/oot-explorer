use oot_explorer_core::fs::{LazyFileSystem, VromAddr};
use oot_explorer_core::rom::Rom;
use oot_explorer_core::versions::oot_ntsc_10;
use scoped_owner::ScopedOwner;
use std::fmt::Write;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{
    Document, HtmlElement, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition,
};

use crate::{Context, InnerContext};

const ROW_HEIGHT: u32 = 14;

/// Pixel span to extend the viewport to reduce the incidence of missing hexdump rows while
/// scrolling.
const RENDER_MARGIN: u32 = 100;

#[wasm_bindgen]
pub struct HexDumpView {
    document: Document,
    _ctx: Arc<Mutex<InnerContext>>,
    element: HtmlElement,
    highlight: Option<Range<VromAddr>>,
    base_addr: VromAddr,
    data: Box<[u8]>,
    prev_viewport: Option<Range<u32>>,
    rows: Vec<(Range<VromAddr>, HtmlElement)>,
}

#[wasm_bindgen]
impl HexDumpView {
    #[wasm_bindgen(constructor)]
    pub fn new(document: &Document, ctx: &Context) -> HexDumpView {
        let element = html_template!(document,
            return div[class="hexdump"] {}
        );

        // TODO: Remove this arbitrary choice!
        element
            .style()
            .set_property("height", "48846px")
            .unwrap_throw();

        // TODO: Remove this arbitrary choice. Make the caller provide data and a base address.
        let (base_addr, data) = ScopedOwner::with_scope(|scope| {
            let ref_ctx = ctx.inner.lock().unwrap_throw();
            let mut fs = LazyFileSystem::new(
                Rom::new(&ref_ctx.rom_data),
                oot_ntsc_10::FILE_TABLE_ROM_ADDR,
            );
            let scene_table = oot_ntsc_10::get_scene_table(scope, &mut fs);
            let scene = scene_table
                .iter()
                .next()
                .unwrap_throw()
                .scene(scope, &mut fs);
            let base_addr = scene.addr();
            let data = scene.data().to_owned().into_boxed_slice();
            (base_addr, data)
        });

        // for offset in (0..0x0000da10).step_by(0x10) {
        //     let row = make_row(document, &data[offset as usize..], base_addr + offset, None);
        //     element.append_child(&row).unwrap_throw();
        // }

        HexDumpView {
            document: document.clone(),
            _ctx: Arc::clone(&ctx.inner),
            element,
            highlight: None,
            base_addr,
            data,
            prev_viewport: None,
            rows: vec![],
        }
    }

    #[wasm_bindgen(js_name = regenerateChildren)]
    pub fn regenerate_children(&mut self) {
        self.regenerate_children_with_force(false);
    }

    fn regenerate_children_with_force(&mut self, force: bool) {
        // Compute the current viewport.
        let parent = self.element.parent_element().unwrap_throw();
        let viewport = {
            let y_start = parent.scroll_top() as u32;
            let y_end = y_start + parent.get_bounding_client_rect().height() as u32;

            y_start.saturating_sub(RENDER_MARGIN)..y_end.saturating_add(RENDER_MARGIN)
        };

        // Bail if the viewport hasn't changed.
        if let Some(prev_viewport) = self.prev_viewport.as_ref() {
            if viewport == *prev_viewport && !force {
                return;
            }
        }
        self.prev_viewport = Some(viewport.clone());

        // Remove all rows.
        while let Some(child) = self.element.first_child() {
            self.element.remove_child(&child).unwrap_throw();
        }
        self.rows.clear();

        // Create all visible rows.
        let first_row_index = viewport.start / ROW_HEIGHT;
        let mut row_top = first_row_index * ROW_HEIGHT;
        for row_index in first_row_index.. {
            // Stop when the next row would not be visible.
            if row_top >= viewport.end {
                break;
            }
            let row_bottom = row_top + ROW_HEIGHT;

            let offset = row_index * 16;
            let addr = self.base_addr + offset;
            let data = match self.data.get(offset as usize..) {
                Some(data) => data,
                None => &[],
            };

            let row = make_row(&self.document, data, addr, self.highlight.as_ref());
            self.element.append_child(&row).unwrap_throw();
            self.rows.push((addr..(addr + 16), row.clone()));

            row.style()
                .set_property("top", &format!("{}px", row_top))
                .unwrap_throw();

            row_top = row_bottom;
        }
    }

    #[wasm_bindgen(getter)]
    pub fn element(&self) -> HtmlElement {
        self.element.clone()
    }

    #[wasm_bindgen(js_name = setHighlight)]
    pub fn js_set_highlight(&mut self, start: u32, end: u32) {
        self.set_highlight(Some(VromAddr(start)..VromAddr(end)));
    }

    #[wasm_bindgen(js_name = clearHighlight)]
    pub fn js_clear_highlight(&mut self) {
        self.set_highlight(None);
    }

    fn set_highlight(&mut self, highlight: Option<Range<VromAddr>>) {
        if highlight == self.highlight {
            return;
        }

        self.highlight = highlight;
        self.regenerate_children_with_force(true);

        // TODO: Update a smaller set of rows.

        // for (offset, old_row) in (0..0x0000da10).step_by(0x10).zip(self.rows.iter_mut()) {
        //     let addr = self.base_addr + offset;
        //     let row_range = addr..addr + 16;
        //     let old_rel = range_relation(self.highlight.as_ref(), &row_range);
        //     let new_rel = range_relation(highlight.as_ref(), &row_range);
        //     if old_rel != new_rel || new_rel == RangeRelation::CrossesReference {
        //         // The highlight change might affect this row.
        //         let new_row = make_row(
        //             &self.document,
        //             &self.data[offset as usize..],
        //             addr,
        //             highlight.as_ref(),
        //         );

        //         // Replace the row element by inserting the new element before the old one and then
        //         // removing the old one.
        //         self.element
        //             .insert_before(&new_row, Some(&old_row.element))
        //             .unwrap_throw();
        //         self.element.remove_child(&old_row.element).unwrap_throw();

        //         // Finally, overwrite the old row.
        //         *old_row = new_row;
        //     }
        // }
    }

    #[wasm_bindgen(js_name = scrollToAddr)]
    pub fn scroll_to_addr(&mut self, addr: u32) {
        let addr = VromAddr(addr);
        for row in &self.rows {
            if row.0.contains(&addr) {
                row.1.scroll_into_view_with_scroll_into_view_options(&{
                    let mut options = ScrollIntoViewOptions::new();
                    options
                        .behavior(ScrollBehavior::Smooth)
                        .block(ScrollLogicalPosition::Center);
                    options
                });
                return;
            }
        }

        // No hits. The target element must be out of the viewport. Construct a temporary fake row
        // and scroll to it.
        let row_top = (addr - self.base_addr) / 16 * ROW_HEIGHT;
        let fake_row = html_template!(&self.document, return div[class="hexdump-row"] {});
        self.element.append_child(&fake_row).unwrap_throw();
        fake_row
            .style()
            .set_property("top", &format!("{}px", row_top))
            .unwrap_throw();
        fake_row.scroll_into_view_with_scroll_into_view_options(&{
            let mut options = ScrollIntoViewOptions::new();
            options
                .behavior(ScrollBehavior::Smooth)
                .block(ScrollLogicalPosition::Center);
            options
        });
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)] // I intend to reuse this eventually.
enum RangeRelation {
    OutsideReference,
    CrossesReference,
    InsideReference,
}

#[allow(dead_code)] // I intend to reuse this eventually.
fn range_relation(reference: Option<&Range<VromAddr>>, test: &Range<VromAddr>) -> RangeRelation {
    match reference {
        Some(reference) => {
            if test.end <= reference.start || reference.end <= test.start {
                RangeRelation::OutsideReference
            } else if reference.start <= test.start && test.end <= reference.end {
                RangeRelation::InsideReference
            } else {
                RangeRelation::CrossesReference
            }
        }
        None => RangeRelation::OutsideReference,
    }
}

fn make_row(
    document: &Document,
    data: &[u8],
    addr: VromAddr,
    highlight: Option<&Range<VromAddr>>,
) -> HtmlElement {
    let element = html_template!(document, return div[class="hexdump-row"] {});

    // Start with the address.
    let mut text = format!("{:08x}", addr.0);
    let flush = |text: &mut String, in_highlight| {
        if !text.is_empty() {
            if in_highlight {
                html_template!(document, in element:
                    span[class="hexdump-highlight"] { text(&text) }
                );
            } else {
                html_template!(document, in element: text(&text));
            }
            text.clear();
        }
    };

    // Append each byte with a pattern of spaces between.
    let mut in_highlight = false;
    for i in 0..16 {
        // Big spaces every four bytes, including before the first one.
        write!(&mut text, "{}", if (i & 3) == 0 { "  " } else { " " },).unwrap_throw();

        // Start highlighting if it's time.
        if let Some(highlight) = highlight {
            if !in_highlight && highlight.contains(&(addr + i)) {
                flush(&mut text, false);
                in_highlight = true;
            }
        }

        // Append the byte's value, if accessible, or a placeholder.
        match data.get(i as usize) {
            Some(x) => write!(&mut text, "{:02x}", x).unwrap_throw(),
            None => write!(&mut text, "--").unwrap_throw(),
        }

        // End highlighting if it's time.
        if let Some(highlight) = highlight {
            if in_highlight && !highlight.contains(&(addr + i + 1)) {
                flush(&mut text, true);
                in_highlight = false;
            }
        }
    }

    flush(&mut text, in_highlight);
    element
}
