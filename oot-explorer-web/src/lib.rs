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
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlSampler, WebGlTexture};

mod sampler_cache;
mod texture_cache;

use sampler_cache::SamplerCache;
use texture_cache::TextureCache;

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct Context {
    gl: WebGl2RenderingContext,
    rom_data: Box<[u8]>,
    texture_cache: TextureCache,
    sampler_cache: SamplerCache,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessSceneBatch<'a> {
    fragment_shader: &'a str,
    #[serde(with = "serde_bytes")]
    vertex_data: &'a [u8],
    translucent: bool,
    textures: Vec<ProcessSceneTexture>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessSceneTexture {
    texture_key: u32,
    sampler_key: u32,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(gl: WebGl2RenderingContext, rom_data: Box<[u8]>) -> Context {
        Context {
            gl,
            rom_data,
            texture_cache: TextureCache::new(),
            sampler_cache: SamplerCache::new(),
        }
    }

    #[wasm_bindgen(js_name = "processScene")]
    pub fn process_scene(&mut self, scene_index: usize) -> JsValue {
        let Context {
            ref gl,
            ref rom_data,
            ref mut texture_cache,
            ref mut sampler_cache,
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

            let mut batches = vec![];
            for batch in dlist_interp.iter_batches() {
                let mut textures = vec![];
                for texture_state in &batch.textures {
                    // Cache all referenced textures and samplers.
                    texture_cache.get_or_decode(gl, scope, &mut fs, &texture_state.descriptor);
                    sampler_cache.get_or_create(gl, &texture_state.params);

                    textures.push(ProcessSceneTexture {
                        texture_key: texture_cache::opaque_key(&texture_state.descriptor),
                        sampler_key: sampler_cache::opaque_key(&texture_state.params),
                        width: texture_state.descriptor.render_width as u32,
                        height: texture_state.descriptor.render_height as u32,
                    });
                }

                batches.push(ProcessSceneBatch {
                    fragment_shader: &batch.fragment_shader,
                    vertex_data: &batch.vertex_data,
                    translucent: batch.translucent,
                    textures,
                });
            }

            serde_wasm_bindgen::to_value(&batches).unwrap()
        })
    }

    #[wasm_bindgen(js_name = "getTexture")]
    pub fn get_texture(&self, key: u32) -> Option<WebGlTexture> {
        self.texture_cache.get_with_key(key).cloned()
    }

    #[wasm_bindgen(js_name = "getSampler")]
    pub fn get_sampler(&self, key: u32) -> Option<WebGlSampler> {
        self.sampler_cache.get_with_key(key).cloned()
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
                            dlist_interp.interpret(&rsp_ctx, false, dlist);
                        }
                        if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, true, dlist);
                        }
                    }
                }
                MeshVariant::Jfif(_) => (),
                MeshVariant::Clipped(mesh) => {
                    for entry in mesh.entries(&cpu_ctx) {
                        if let Ok(Some(dlist)) = entry.opaque_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, false, dlist);
                        }
                        if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, true, dlist);
                        }
                    }
                }
            };
        }
    }
}
