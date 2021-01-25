use oot_explorer_core::fs::{LazyFileSystem, VromAddr};
use oot_explorer_core::rom::Rom;
use oot_explorer_core::scene::SCENE_DESC;
use oot_explorer_core::segment::{Segment, SegmentCtx};
use oot_explorer_core::versions::oot_ntsc_10;
use scoped_owner::ScopedOwner;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, HtmlElement};

use crate::explore::reflect::item::{ItemView, Nesting};
use crate::InnerContext;

mod item;

pub struct ReflectView {
    element: HtmlElement,
    root_item: ItemView,
}

impl ReflectView {
    pub fn new(document: &Document, ctx: &Arc<Mutex<InnerContext>>) -> ReflectView {
        let element = html_template!(document,
            return div[class="explore-view-tree"] {}
        );

        let ctx_ref = ctx.lock().unwrap_throw();
        let root_item = ScopedOwner::with_scope(|scope| {
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

            let mut root_item = ItemView::for_value(
                document,
                Arc::clone(ctx),
                Nesting::default(),
                scope,
                &mut fs,
                &segment_ctx,
                scene.addr(),
                SCENE_DESC,
            );
            element.append_child(&*root_item.element()).unwrap_throw();
            root_item.expand(document, scope, &mut fs, &segment_ctx);
            root_item
        });

        ReflectView { element, root_item }
    }

    pub fn element(&self) -> &HtmlElement {
        &self.element
    }

    pub fn set_highlight_listener(
        &mut self,
        listener: Option<Arc<dyn Fn(Option<Range<VromAddr>>)>>,
    ) {
        self.root_item.set_highlight_listener(listener);
    }
}
