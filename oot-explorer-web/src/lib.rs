use js_sys::{Array, Object, Reflect};
use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::header::{RoomHeader, SceneHeader};
use oot_explorer_core::mesh::MeshVariant;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::room::Room;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx};
use oot_explorer_core::versions;
use oot_explorer_gl::display_list_interpreter::DisplayListInterpreter;
use scoped_owner::ScopedOwner;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

mod texture_cache;

use texture_cache::TextureCache;

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

fn new_array_buffer(data: &[u8]) -> js_sys::ArrayBuffer {
    let start = &data[0] as *const u8 as usize;
    let end = start + data.len();
    let src = js_sys::Uint8Array::new(
        &wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap_throw()
            .buffer()
            .dyn_into::<js_sys::ArrayBuffer>()
            .unwrap_throw(),
    )
    .slice(
        start.try_into().unwrap_throw(),
        end.try_into().unwrap_throw(),
    );

    let dst_buffer = js_sys::ArrayBuffer::new(data.len().try_into().unwrap_throw());
    let dst_typed_array = js_sys::Uint8Array::new(&dst_buffer);
    dst_typed_array.set(&src, 0);
    dst_buffer
}

#[wasm_bindgen]
pub struct Context {
    gl: WebGl2RenderingContext,
    rom_data: Box<[u8]>,
    texture_cache: TextureCache,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(gl: WebGl2RenderingContext, rom_data: Box<[u8]>) -> Context {
        Context {
            gl,
            rom_data,
            texture_cache: TextureCache::new(),
        }
    }

    #[wasm_bindgen(js_name = "processScene")]
    pub fn process_scene(&mut self, scene_index: usize) -> Array {
        let Context {
            ref gl,
            ref rom_data,
            ref mut texture_cache,
        } = self;
        let rom = Rom::new(rom_data);
        ScopedOwner::with_scope(|scope| {
            let mut fs = LazyFileSystem::new(rom, versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR);
            let mut dlist_interp = DisplayListInterpreter::new();

            let scene = versions::oot_ntsc_10::get_scene_table(scope, &mut fs)
                .get(scene_index)
                .scene(scope, &mut fs);
            examine_scene(scope, &mut fs, scene, &mut dlist_interp);

            // TODO: Set up a web-friendly logger of some kind.
            println!("total_dlists: {}", dlist_interp.total_dlists());
            println!("total_instructions: {}", dlist_interp.total_instructions());
            println!("unmapped_calls: {:?}", dlist_interp.unmapped_calls());
            println!("max_depth: {}", dlist_interp.max_depth());
            println!("total_lit_verts: {}", dlist_interp.total_lit_verts());
            println!("total_unlit_verts: {}", dlist_interp.total_unlit_verts());

            let js_result = Array::new();
            for batch in dlist_interp.iter_batches() {
                // Fetch all referenced textures into the texture cache.
                for descriptor in &batch.textures {
                    texture_cache.get_or_decode(gl, scope, &mut fs, descriptor);
                }

                let js_batch = Object::new();

                Reflect::set(
                    &js_batch,
                    &JsValue::from_str("fragmentShader"),
                    &JsValue::from_str(&batch.fragment_shader),
                )
                .unwrap_throw();

                Reflect::set(
                    &js_batch,
                    &JsValue::from_str("vertexData"),
                    &new_array_buffer(&batch.vertex_data),
                )
                .unwrap_throw();

                let js_textures = Array::new();
                for texture in &batch.textures {
                    let js_texture = Object::new();
                    js_textures.push(&js_texture);

                    Reflect::set(
                        &js_texture,
                        &JsValue::from_str("key"),
                        &texture_cache::opaque_key(texture).into(),
                    )
                    .unwrap_throw();

                    Reflect::set(
                        &js_texture,
                        &JsValue::from_str("width"),
                        &(texture.render_width as u32).into(),
                    )
                    .unwrap_throw();

                    Reflect::set(
                        &js_texture,
                        &JsValue::from_str("height"),
                        &(texture.render_height as u32).into(),
                    )
                    .unwrap_throw();
                }

                Reflect::set(&js_batch, &JsValue::from_str("textures"), &js_textures)
                    .unwrap_throw();

                js_result.push(&js_batch);
            }

            js_result
        })
    }

    #[wasm_bindgen(js_name = "getTexture")]
    pub fn get_texture(&self, key: u32) -> Option<WebGlTexture> {
        self.texture_cache.get_with_key(key).cloned()
    }
}

fn examine_scene<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    scene: Scene<'a>,
    dlist_interp: &mut DisplayListInterpreter,
) {
    let ctx = {
        let mut ctx = SegmentCtx::new();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx
    };
    for header in scene.headers() {
        if let SceneHeader::RoomList(header) = header {
            for room_list_entry in header.room_list(&ctx) {
                let room = room_list_entry.room(scope, fs);
                examine_room(scope, fs, scene, room, dlist_interp);
            }
        }
    }
}

fn examine_room<'a>(
    _scope: &'a ScopedOwner,
    _fs: &mut LazyFileSystem<'a>,
    scene: Scene<'a>,
    room: Room<'a>,
    dlist_interp: &mut DisplayListInterpreter,
) {
    let cpu_ctx = {
        let mut ctx = SegmentCtx::new();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx.set(Segment::ROOM, room.data(), room.vrom_range());
        ctx
    };
    let rsp_ctx = {
        let ctx = cpu_ctx.clone();

        // const ICON_ITEM_STATIC: usize = 8;
        // ctx.set(
        //     Segment(8),
        //     fs.get_file(scope, ICON_ITEM_STATIC),
        //     fs.metadata(ICON_ITEM_STATIC).virtual_range(),
        // );
        // const ICON_ITEM_24_STATIC: usize = 8;
        // ctx.set(
        //     Segment(9),
        //     fs.get_file(scope, ICON_ITEM_24_STATIC),
        //     fs.metadata(ICON_ITEM_24_STATIC).virtual_range(),
        // );

        ctx
    };
    for header in room.headers() {
        if let RoomHeader::Mesh(header) = header {
            match header.mesh(&cpu_ctx).variant() {
                MeshVariant::Simple(mesh) => {
                    for entry in mesh.entries(&cpu_ctx) {
                        if let Ok(Some(dlist)) = entry.opaque_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, dlist);
                        }
                        if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, dlist);
                        }
                    }
                }
                MeshVariant::Jfif(_) => (),
                MeshVariant::Clipped(mesh) => {
                    for entry in mesh.entries(&cpu_ctx) {
                        if let Ok(Some(dlist)) = entry.opaque_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, dlist);
                        }
                        if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, dlist);
                        }
                    }
                }
            };
        }
    }
}
