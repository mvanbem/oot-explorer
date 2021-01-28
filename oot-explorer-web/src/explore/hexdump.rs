use oot_explorer_core::fs::{LazyFileSystem, VromAddr};
use oot_explorer_core::rom::Rom;
use oot_explorer_core::versions::oot_ntsc_10;
use scoped_owner::ScopedOwner;
use std::fmt::Write;
use std::ops::Range;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, HtmlElement};

use crate::Context;

#[wasm_bindgen]
pub struct HexDumpView {
    document: Document,
    element: HtmlElement,
    highlight: Option<Range<VromAddr>>,
    rows: Vec<Row>,
    base_addr: VromAddr,
    data: Box<[u8]>,
}

#[wasm_bindgen]
impl HexDumpView {
    #[wasm_bindgen(constructor)]
    pub fn new(document: &Document, ctx: &Context) -> HexDumpView {
        let element = html_template!(document,
            return div[class="explore-view-hexdump"] {}
        );

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

        let mut rows = vec![];
        for offset in (0..0x0000da10).step_by(0x10) {
            let row = Row::new(document, &data[offset as usize..], base_addr + offset, None);
            element.append_child(&row.element).unwrap_throw();
            rows.push(row);
        }

        HexDumpView {
            document: document.clone(),
            element,
            highlight: None,
            rows,
            base_addr,
            data,
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

        for (offset, old_row) in (0..0x0000da10).step_by(0x10).zip(self.rows.iter_mut()) {
            let addr = self.base_addr + offset;
            let row_range = addr..addr + 16;
            let old_rel = range_relation(self.highlight.as_ref(), &row_range);
            let new_rel = range_relation(highlight.as_ref(), &row_range);
            if old_rel != new_rel || new_rel == RangeRelation::CrossesReference {
                // The highlight change might affect this row.
                let new_row = Row::new(
                    &self.document,
                    &self.data[offset as usize..],
                    addr,
                    highlight.as_ref(),
                );

                // Replace the row element by inserting the new element before the old one and then
                // removing the old one.
                self.element
                    .insert_before(&new_row.element, Some(&old_row.element))
                    .unwrap_throw();
                self.element.remove_child(&old_row.element).unwrap_throw();

                // Finally, overwrite the old row.
                *old_row = new_row;
            }
        }

        self.highlight = highlight;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RangeRelation {
    OutsideReference,
    CrossesReference,
    InsideReference,
}

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

struct Row {
    element: HtmlElement,
}

impl Row {
    fn new(
        document: &Document,
        data: &[u8],
        addr: VromAddr,
        highlight: Option<&Range<VromAddr>>,
    ) -> Row {
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
        Row { element }
    }
}
