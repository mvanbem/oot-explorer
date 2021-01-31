#![cfg_attr(feature = "trace_macros", feature(trace_macros))]

#[cfg(feature = "trace_macros")]
trace_macros!(true);

use oot_explorer_core::fs::{fully_decompress, FileTable, LazyFileSystem, OwnedVrom, Vrom};
use oot_explorer_core::header::room::RoomHeaderVariant;
use oot_explorer_core::header::scene::SceneHeaderVariant;
use oot_explorer_core::mesh::{
    Background, ClippedMeshEntry, JfifMeshVariant, MeshVariant, SimpleMeshEntry,
};
use oot_explorer_core::reflect::sourced::RangeSourced;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::room::Room;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentTable};
use oot_explorer_core::versions;
use oot_explorer_gl::display_list_interpreter::DisplayListInterpreter;
use scoped_owner::ScopedOwner;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlSampler, WebGlTexture};

#[macro_use]
mod macros;

mod explore;
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
    inner: Arc<Mutex<InnerContext>>,
}
pub struct InnerContext {
    gl: WebGl2RenderingContext,
    rom_data: Box<[u8]>,
    file_table: Option<FileTable>,
    vrom: Option<OwnedVrom>,
    texture_cache: TextureCache,
    sampler_cache: SamplerCache,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessSceneResult<'a> {
    batches: Vec<ProcessSceneBatch<'a>>,
    backgrounds: Vec<String>,
    start_pos: Option<[f64; 5]>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessSceneBatch<'a> {
    fragment_shader: &'a str,
    #[serde(with = "serde_bytes")]
    vertex_data: &'a [u8],
    translucent: bool,
    textures: Vec<ProcessSceneTexture>,
    z_upd: bool,
    decal: bool,
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
            inner: Arc::new(Mutex::new(InnerContext {
                gl,
                rom_data,
                file_table: None,
                vrom: None,
                texture_cache: TextureCache::new(),
                sampler_cache: SamplerCache::new(),
            })),
        }
    }

    #[wasm_bindgen(js_name = decompress)]
    pub fn decompress(&self) {
        let mut inner_mut = self.inner.lock().unwrap_throw();
        if inner_mut.file_table.is_none() || inner_mut.vrom.is_none() {
            let (vrom, file_table) = fully_decompress(
                Rom::new(&inner_mut.rom_data),
                versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR,
            );
            inner_mut.vrom = Some(vrom);
            inner_mut.file_table = Some(file_table);
        }
        let mut vrom_data = vec![];
        ScopedOwner::with_scope(|scope| {
            let mut fs = LazyFileSystem::new(
                Rom::new(&inner_mut.rom_data),
                versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR,
            );
            for file_index in 0..fs.len() {
                let (start, end) = {
                    let range = fs.metadata(file_index).virtual_range();
                    (range.start.0 as usize, range.end.0 as usize)
                };
                if vrom_data.len() < end {
                    vrom_data.resize(end, 0x00);
                }
                (&mut vrom_data[start..end]).copy_from_slice(fs.get_file(scope, file_index));
            }
        });
        inner_mut.vrom = Some(vrom_data.into_boxed_slice());
    }

    #[wasm_bindgen(js_name = processScene)]
    pub fn process_scene(&self, scene_index: usize) -> JsValue {
        let mut inner_mut = self.inner.lock().unwrap_throw();
        let InnerContext {
            ref gl,
            ref rom_data,
            ref file_table,
            ref vrom,
            ref mut texture_cache,
            ref mut sampler_cache,
        } = *inner_mut;
        let rom = Rom::new(rom_data);
        let file_table = file_table.as_ref().unwrap_throw();
        let vrom = vrom.as_ref().unwrap_throw().borrow();

        ScopedOwner::with_scope(|scope| {
            let scene = versions::oot_ntsc_10::get_scene_table(file_table, vrom)
                .get(scene_index)
                .scene(vrom);
            let mut dlist_interp = DisplayListInterpreter::new();
            let mut backgrounds = vec![];
            let start_pos = examine_scene(vrom, scene, &mut dlist_interp, &mut backgrounds);

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
                    z_upd: batch.z_upd,
                    decal: batch.decal,
                });
            }

            serde_wasm_bindgen::to_value(&ProcessSceneResult {
                batches,
                backgrounds,
                start_pos,
            })
            .unwrap_throw()
        })
    }

    #[wasm_bindgen(getter = sceneCount)]
    pub fn scene_count(&self) -> u32 {
        versions::oot_ntsc_10::SCENE_TABLE_COUNT as u32
    }

    #[wasm_bindgen(js_name = getTexture)]
    pub fn get_texture(&self, key: u32) -> Option<WebGlTexture> {
        self.inner
            .lock()
            .unwrap_throw()
            .texture_cache
            .get_with_key(key)
            .cloned()
    }

    #[wasm_bindgen(js_name = getSampler)]
    pub fn get_sampler(&self, key: u32) -> Option<WebGlSampler> {
        self.inner
            .lock()
            .unwrap_throw()
            .sampler_cache
            .get_with_key(key)
            .cloned()
    }
}

fn examine_scene(
    vrom: Vrom<'_>,
    scene: RangeSourced<Scene<'_>>,
    dlist_interp: &mut DisplayListInterpreter,
    backgrounds: &mut Vec<String>,
) -> Option<[f64; 5]> {
    let ctx = {
        let mut ctx = SegmentTable::new();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx
    };
    let mut start_pos = None;
    for header in scene.headers() {
        match header.variant() {
            SceneHeaderVariant::StartPositions(header) => {
                start_pos = header.start_positions(&ctx).iter().next().map(|actor| {
                    [
                        actor.pos_x() as f64,
                        actor.pos_y() as f64,
                        actor.pos_z() as f64,
                        actor.angle_x() as f64 * std::f64::consts::TAU / 65536.0,
                        actor.angle_y() as f64 * std::f64::consts::TAU / 65536.0,
                    ]
                });
            }
            SceneHeaderVariant::RoomList(header) => {
                for room_list_entry in header.room_list(&ctx) {
                    let room = room_list_entry.room(scope, fs);
                    examine_room(scene, room, dlist_interp, backgrounds);
                }
            }
            _ => (),
        }
    }
    start_pos
}

fn examine_room(
    vrom: Vrom<'_>,
    scene: RangeSourced<Scene<'_>>,
    room: RangeSourced<Room<'_>>,
    dlist_interp: &mut DisplayListInterpreter,
    backgrounds: &mut Vec<String>,
) {
    let cpu_ctx = {
        let mut ctx = SegmentTable::new();
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

    let handle_simple_mesh_entry = |dlist_interp: &mut DisplayListInterpreter,
                                    entry: SimpleMeshEntry<'a>| {
        if let Ok(Some(dlist)) = entry.opaque_display_list(&cpu_ctx) {
            dlist_interp.interpret(&rsp_ctx, false, dlist);
        }
        if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
            dlist_interp.interpret(&rsp_ctx, true, dlist);
        }
    };

    let handle_clipped_mesh_entry =
        |dlist_interp: &mut DisplayListInterpreter, entry: ClippedMeshEntry<'a>| {
            if let Ok(Some(dlist)) = entry.opaque_display_list(&cpu_ctx) {
                dlist_interp.interpret(&rsp_ctx, false, dlist);
            }
            if let Ok(Some(dlist)) = entry.translucent_display_list(&cpu_ctx) {
                dlist_interp.interpret(&rsp_ctx, true, dlist);
            }
        };

    let background_to_string = |background: Background<'a>| {
        // TODO: Could definitely pre-allocate here instead of growing incrementally.
        let mut result = "data:image/jpeg;base64,".to_string();
        let data = cpu_ctx.resolve(background.ptr()).unwrap_throw();
        base64::encode_config_buf(data, base64::STANDARD_NO_PAD, &mut result);
        result
    };

    for header in room.headers() {
        if let RoomHeaderVariant::Mesh(header) = header.variant() {
            match header.mesh(&cpu_ctx).variant() {
                MeshVariant::Simple(mesh) => {
                    for entry in mesh.entries(&cpu_ctx) {
                        handle_simple_mesh_entry(dlist_interp, entry);
                    }
                }
                MeshVariant::Jfif(mesh) => match mesh.variant() {
                    JfifMeshVariant::Single(mesh) => {
                        handle_simple_mesh_entry(dlist_interp, mesh.mesh_entry(&cpu_ctx));
                        backgrounds.push(background_to_string(mesh.background()));
                    }
                    JfifMeshVariant::Multiple(mesh) => {
                        handle_simple_mesh_entry(dlist_interp, mesh.mesh_entry(&cpu_ctx));
                        for entry in mesh.background_entries(&cpu_ctx) {
                            backgrounds.push(background_to_string(entry.background()));
                        }
                    }
                },
                MeshVariant::Clipped(mesh) => {
                    for entry in mesh.entries(&cpu_ctx) {
                        handle_clipped_mesh_entry(dlist_interp, entry);
                    }
                }
            };
        }
    }
}
