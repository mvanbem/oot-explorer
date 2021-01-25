use oot_explorer_core::fs::VromAddr;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{window, Document, HtmlElement};

use crate::explore::hexdump::HexDumpView;
use crate::explore::reflect::ReflectView;
use crate::Context;

mod hexdump;
mod reflect;

#[wasm_bindgen]
pub struct ExploreView {
    inner: Arc<Mutex<InnerExploreView>>,
}

#[wasm_bindgen]
impl ExploreView {
    pub fn new(document: &Document, ctx: &Context) -> ExploreView {
        let element = html_template!(document,
            return div[class="explore-view"] {
                div[class="explore-view-title-bar"] {
                    div[class="explore-view-title"] { text("Current Scene") }
                    let close_button = div[class="explore-view-close"] {
                        // U+2573 BOX DRAWINGS LIGHT DIAGONAL CROSS
                        text("\u{2573}")
                    }
                }
            }
        );

        let hexdump = HexDumpView::new(document, &ctx.inner);
        element.append_child(hexdump.element()).unwrap_throw();

        let reflect = ReflectView::new(document, &ctx.inner);
        element.append_child(reflect.element()).unwrap_throw();

        let inner = Arc::new(Mutex::new(InnerExploreView {
            element,
            hexdump,
            reflect: reflect,
        }));
        let mut inner_mut = inner.lock().unwrap_throw();
        inner_mut.reflect.set_highlight_listener(Some(Arc::new({
            let weak_inner = Arc::downgrade(&inner);
            move |range| {
                if let Some(inner) = weak_inner.upgrade() {
                    inner.lock().unwrap_throw().handle_highlight(range);
                }
            }
        })));
        drop(inner_mut);

        ExploreView { inner }
    }

    #[wasm_bindgen(getter = element)]
    pub fn element(&self) -> HtmlElement {
        self.inner.lock().unwrap_throw().element.clone()
    }
}

struct InnerExploreView {
    element: HtmlElement,
    hexdump: HexDumpView,
    reflect: ReflectView,
}

impl InnerExploreView {
    fn handle_highlight(&mut self, highlight: Option<Range<VromAddr>>) {
        self.hexdump.set_highlight(
            &window().unwrap_throw().document().unwrap_throw(),
            highlight,
        );
    }
}
