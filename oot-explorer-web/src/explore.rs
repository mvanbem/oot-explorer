use oot_explorer_core::fs::VromAddr;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Document, HtmlElement, MouseEvent};

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
        html_template!(document,
            let element = div[class="explore-view"] {
                div[class="explore-view-title-bar"] {
                    let title = div[class="explore-view-title"] { text("Current Scene") }
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
            title_mousedown_handler: None,
        }));
        let mut inner_mut = inner.lock().unwrap_throw();

        // Attach a mousedown event handler to the title bar.
        inner_mut.title_mousedown_handler = Some(Closure::wrap(Box::new({
            let weak_inner = Arc::downgrade(&inner);
            move |event| {
                if let Some(inner) = weak_inner.upgrade() {
                    inner.lock().unwrap_throw().handle_title_mousedown(event);
                }
            }
        })
            as Box<dyn Fn(MouseEvent)>));
        title
            .add_event_listener_with_callback(
                "mousedown",
                inner_mut
                    .title_mousedown_handler
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap_throw();

        // Attach a highlight event handler to the tree view.
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
    title_mousedown_handler: Option<Closure<dyn Fn(MouseEvent)>>,
}

impl InnerExploreView {
    fn handle_title_mousedown(&mut self, event: MouseEvent) {
        web_sys::console::log_1(&"title mousedown".into());
    }

    fn handle_highlight(&mut self, highlight: Option<Range<VromAddr>>) {
        self.hexdump.set_highlight(
            &window().unwrap_throw().document().unwrap_throw(),
            highlight,
        );
    }
}
