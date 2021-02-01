use oot_explorer_vrom::VromAddr;
use std::fmt::Write;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{
    Document, HtmlElement, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition,
};

use crate::reflect_root::ReflectRoot;
use crate::{Context, InnerContext};

const ROW_HEIGHT: u32 = 14;

/// Pixel span to extend the viewport to reduce the incidence of missing hexdump rows while
/// scrolling.
const RENDER_MARGIN: u32 = 100;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Marking {
    None,
    Selection,
    Highlight,
}

impl Marking {
    fn only_if(self, condition: bool) -> Marking {
        if condition {
            self
        } else {
            Marking::None
        }
    }
}

#[wasm_bindgen]
pub struct HexDumpView {
    document: Document,
    ctx: Arc<Mutex<InnerContext>>,
    root: ReflectRoot,
    element: HtmlElement,
    rows: Vec<(Range<VromAddr>, HtmlElement)>,
    markings: Vec<(Marking, Range<VromAddr>)>,
}

#[wasm_bindgen]
impl HexDumpView {
    #[wasm_bindgen(constructor)]
    pub fn new(document: &Document, ctx: &Context, root: &ReflectRoot) -> HexDumpView {
        let element = html_template!(document,
            return div[class="hexdump"] {}
        );

        // The root element's contents are ephemeral and follow the scroll position, so set a fixed
        // height.
        let row_count = (root.vrom_range.end - root.vrom_range.start + 15) / 16 * ROW_HEIGHT;
        element
            .style()
            .set_property("height", &format!("{}px", row_count))
            .unwrap_throw();

        HexDumpView {
            document: document.clone(),
            ctx: Arc::clone(&ctx.inner),
            root: root.clone(),
            element,
            rows: vec![],
            markings: vec![],
        }
    }

    #[wasm_bindgen(js_name = regenerateChildren)]
    pub fn regenerate_children(&mut self) {
        // Get a reference to the data.
        let ctx = self.ctx.lock().unwrap_throw();
        let vrom = ctx.vrom.as_ref().unwrap_throw().borrow();
        let data = vrom.slice(self.root.vrom_range.clone()).unwrap_throw();

        // Compute the current viewport.
        let parent = self.element.parent_element().unwrap_throw();
        let viewport = {
            let y_start = parent.scroll_top() as u32;
            let y_end = y_start + parent.get_bounding_client_rect().height() as u32;

            y_start.saturating_sub(RENDER_MARGIN)..y_end.saturating_add(RENDER_MARGIN)
        };

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
            let addr = self.root.vrom_range.start + offset;
            let data = match data.get(offset as usize..) {
                Some(data) => data,
                None => continue,
            };

            let row = make_row(&self.document, data, addr, &self.markings);
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

    #[wasm_bindgen(js_name = addHighlight)]
    pub fn add_highlight(&mut self, start: u32, end: u32) {
        self.markings
            .push((Marking::Highlight, VromAddr(start)..VromAddr(end)));
    }

    #[wasm_bindgen(js_name = addSelection)]
    pub fn add_selection(&mut self, start: u32, end: u32) {
        self.markings
            .push((Marking::Selection, VromAddr(start)..VromAddr(end)));
    }

    #[wasm_bindgen(js_name = clearMarkings)]
    pub fn clear_markings(&mut self) {
        self.markings.clear();
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
        let row_top = (addr - self.root.vrom_range.start) / 16 * ROW_HEIGHT;
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

fn make_row(
    document: &Document,
    data: &[u8],
    addr: VromAddr,
    markings: &[(Marking, Range<VromAddr>)],
) -> HtmlElement {
    let element = html_template!(document, return div[class="hexdump-row"] {});

    // Start with the address.
    let mut text = format!("{:08x}", addr.0);

    // This function flushes `text` into the DOM, applying styles for marking as needed.
    let flush = |text: &mut String, marking| {
        if !text.is_empty() {
            match marking {
                Marking::None => {
                    html_template!(document, in element: text(&text));
                }
                Marking::Selection => {
                    html_template!(document, in element:
                        span[class="hexdump-select"] { text(&text) }
                    );
                }
                Marking::Highlight => {
                    html_template!(document, in element:
                        span[class="hexdump-highlight"] { text(&text) }
                    );
                }
            }
            text.clear();
        }
    };

    // Append each byte with a pattern of spaces between.
    let mut current_marking = Marking::None;
    for i in 0..16 {
        // Big spaces every four bytes, including before the first one.
        write!(&mut text, "{}", if (i & 3) == 0 { "  " } else { " " },).unwrap_throw();

        // Start highlighting if it's time.
        let next_marking = markings
            .iter()
            .map(|marking| marking.0.only_if(marking.1.contains(&(addr + i))))
            .max()
            .unwrap_or(Marking::None);
        if next_marking != current_marking {
            flush(&mut text, current_marking);
            current_marking = next_marking;
        }

        // Append the byte's value, if accessible, or a placeholder.
        match data.get(i as usize) {
            Some(x) => write!(&mut text, "{:02x}", x).unwrap_throw(),
            None => write!(&mut text, "--").unwrap_throw(),
        }

        // End highlighting if it's time.
        let next_marking = markings
            .iter()
            .map(|marking| marking.0.only_if(marking.1.contains(&(addr + i + 1))))
            .max()
            .unwrap_or(Marking::None);
        if next_marking != current_marking {
            flush(&mut text, current_marking);
            current_marking = Marking::None;
        }
    }

    flush(&mut text, current_marking);
    element
}
