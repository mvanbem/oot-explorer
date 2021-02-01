#![cfg_attr(feature = "trace_macros", feature(trace_macros))]

#[cfg(feature = "trace_macros")]
trace_macros!(true);

use oot_explorer_game_data::header_room::RoomHeaderVariant;
use oot_explorer_game_data::header_scene::SceneHeaderVariant;
use oot_explorer_game_data::mesh::{
    Background, ClippedMeshEntry, JfifMeshVariant, MeshEntry, MeshVariant, SimpleMeshEntry,
};
use oot_explorer_game_data::room::Room;
use oot_explorer_game_data::scene::Scene;
use oot_explorer_game_data::versions::oot_ntsc_10;
use oot_explorer_gl::display_list_interpreter::{DisplayListInterpreter, DisplayListOpacity};
use oot_explorer_read::VromProxy;
use oot_explorer_rom::OwnedRom;
use oot_explorer_segment::{Segment, SegmentTable};
use oot_explorer_vrom::{decompress, FileIndex, FileTable, OwnedVrom, Vrom};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlSampler, WebGlTexture};

#[macro_use]
mod macros;

mod hexdump;
mod reflect_root;
mod reflect_value;
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
    rom: OwnedRom,
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
                rom: OwnedRom::new(rom_data),
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
            let (file_table, vrom) =
                decompress(inner_mut.rom.borrow(), oot_ntsc_10::FILE_TABLE_ROM_ADDR).unwrap_throw();
            inner_mut.vrom = Some(vrom);
            inner_mut.file_table = Some(file_table);
        }
    }

    #[wasm_bindgen(js_name = processScene)]
    pub fn process_scene(&self, scene_index: u32) -> JsValue {
        let mut inner_mut = self.inner.lock().unwrap_throw();
        let InnerContext {
            ref gl,
            ref file_table,
            ref vrom,
            ref mut texture_cache,
            ref mut sampler_cache,
            ..
        } = *inner_mut;
        let file_table = file_table.as_ref().unwrap_throw();
        let vrom = vrom.as_ref().unwrap_throw().borrow();

        let mut dlist_interp = DisplayListInterpreter::new();
        let mut backgrounds = vec![];
        let start_pos = examine_scene(
            file_table,
            vrom,
            oot_ntsc_10::get_scene_table(file_table)
                .unwrap_throw()
                .get(vrom, scene_index)
                .unwrap_throw()
                .scene(vrom)
                .unwrap_throw()
                .into_inner(),
            &mut dlist_interp,
            &mut backgrounds,
        );

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
                texture_cache.get_or_decode(gl, vrom, &texture_state.descriptor);
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
                translucent: match batch.opacity {
                    DisplayListOpacity::Opaque => false,
                    DisplayListOpacity::Translucent => true,
                },
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
    }

    #[wasm_bindgen(getter = sceneCount)]
    pub fn scene_count(&self) -> u32 {
        oot_ntsc_10::SCENE_TABLE_COUNT as u32
    }

    #[wasm_bindgen(js_name = roomCount)]
    pub fn room_count(&self, scene_index: usize) -> u32 {
        let inner = self.inner.lock().unwrap_throw();
        let file_table = inner.file_table.as_ref().unwrap_throw();
        let vrom = inner.vrom.as_ref().unwrap_throw().borrow();

        let scene_table_entry = oot_ntsc_10::get_scene_table(file_table)
            .unwrap_throw()
            .iter(vrom)
            .nth(scene_index)
            .unwrap_throw()
            .unwrap_throw();
        let scene = scene_table_entry.scene(vrom).unwrap_throw().into_inner();

        scene
            .headers(vrom)
            .find_map(|header| match header.unwrap_throw().variant(vrom) {
                SceneHeaderVariant::RoomList(room_list) => Some(room_list),
                _ => None,
            })
            .unwrap_throw()
            .room_count(vrom) as u32
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
    file_table: &FileTable,
    vrom: Vrom<'_>,
    scene: Scene,
    dlist_interp: &mut DisplayListInterpreter,
    backgrounds: &mut Vec<String>,
) -> Option<[f64; 5]> {
    let segment_table = SegmentTable::new().with(Segment::SCENE, scene.addr());
    let mut start_pos = None;
    for result in scene.headers(vrom) {
        let header = result.unwrap_throw();
        match header.variant(vrom) {
            SceneHeaderVariant::StartPositions(header) => {
                start_pos = header
                    .start_positions(vrom, &segment_table)
                    .unwrap_throw()
                    .iter(vrom)
                    .next()
                    .map(|result| {
                        let actor = result.unwrap_throw();
                        [
                            actor.pos_x(vrom) as f64,
                            actor.pos_y(vrom) as f64,
                            actor.pos_z(vrom) as f64,
                            actor.angle_x(vrom) as f64 * std::f64::consts::TAU / 65536.0,
                            actor.angle_y(vrom) as f64 * std::f64::consts::TAU / 65536.0,
                        ]
                    });
            }
            SceneHeaderVariant::RoomList(header) => {
                for room_list_entry in header
                    .room_list(vrom, &segment_table)
                    .unwrap_throw()
                    .iter(vrom)
                {
                    examine_room(
                        file_table,
                        vrom,
                        scene,
                        room_list_entry
                            .unwrap_throw()
                            .room(vrom)
                            .unwrap_throw()
                            .into_inner(),
                        dlist_interp,
                        backgrounds,
                    );
                }
            }
            _ => (),
        }
    }
    start_pos
}

fn examine_room(
    file_table: &FileTable,
    vrom: Vrom<'_>,
    scene: Scene,
    room: Room,
    dlist_interp: &mut DisplayListInterpreter,
    backgrounds: &mut Vec<String>,
) {
    let cpu_ctx = SegmentTable::new()
        .with(Segment::SCENE, scene.addr())
        .with(Segment::ROOM, room.addr());
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
                                    entry: SimpleMeshEntry| {
        if let Ok(Some(dlist)) = entry.opaque_display_list(vrom, &cpu_ctx) {
            dlist_interp
                .interpret(vrom, &rsp_ctx, DisplayListOpacity::Opaque, dlist)
                .unwrap_throw();
        }
        if let Ok(Some(dlist)) = entry.translucent_display_list(vrom, &cpu_ctx) {
            dlist_interp
                .interpret(vrom, &rsp_ctx, DisplayListOpacity::Translucent, dlist)
                .unwrap_throw();
        }
    };

    let handle_clipped_mesh_entry = |dlist_interp: &mut DisplayListInterpreter,
                                     entry: ClippedMeshEntry| {
        if let Ok(Some(dlist)) = entry.opaque_display_list(vrom, &cpu_ctx) {
            dlist_interp
                .interpret(vrom, &rsp_ctx, DisplayListOpacity::Opaque, dlist)
                .unwrap_throw();
        }
        if let Ok(Some(dlist)) = entry.translucent_display_list(vrom, &cpu_ctx) {
            dlist_interp
                .interpret(vrom, &rsp_ctx, DisplayListOpacity::Translucent, dlist)
                .unwrap_throw();
        }
    };

    let background_to_string = |background: Background| {
        // TODO: Could definitely pre-allocate here instead of growing incrementally.
        let mut result = "data:image/jpeg;base64,".to_string();
        let vrom_start = cpu_ctx.resolve(background.ptr(vrom)).unwrap_throw();

        let mut index = FileIndex(0);
        let vrom_end = loop {
            let range = file_table.file_vrom_range(index).unwrap_throw();
            if range.contains(&vrom_start) {
                break range.end;
            }
            index = FileIndex(index.0 + 1);
        };

        let data = vrom.slice(vrom_start..vrom_end).unwrap_throw();
        base64::encode_config_buf(data, base64::STANDARD_NO_PAD, &mut result);
        result
    };

    for header in room.headers(vrom) {
        if let RoomHeaderVariant::Mesh(header) = header.unwrap_throw().variant(vrom) {
            match header.mesh(vrom, &cpu_ctx).unwrap_throw().variant(vrom) {
                MeshVariant::Simple(mesh) => {
                    for entry in mesh.entries(vrom, &cpu_ctx).unwrap_throw().iter(vrom) {
                        handle_simple_mesh_entry(dlist_interp, entry.unwrap_throw());
                    }
                }
                MeshVariant::Jfif(mesh) => match mesh.variant(vrom) {
                    JfifMeshVariant::Single(mesh) => {
                        handle_simple_mesh_entry(
                            dlist_interp,
                            mesh.mesh_entry(vrom, &cpu_ctx).unwrap_throw(),
                        );
                        backgrounds.push(background_to_string(mesh.background(vrom)));
                    }
                    JfifMeshVariant::Multiple(mesh) => {
                        handle_simple_mesh_entry(
                            dlist_interp,
                            mesh.mesh_entry(vrom, &cpu_ctx).unwrap_throw(),
                        );
                        for entry in mesh
                            .background_entries(vrom, &cpu_ctx)
                            .unwrap_throw()
                            .iter(vrom)
                        {
                            backgrounds
                                .push(background_to_string(entry.unwrap_throw().background(vrom)));
                        }
                    }
                },
                MeshVariant::Clipped(mesh) => {
                    for entry in mesh.entries(vrom, &cpu_ctx).unwrap_throw().iter(vrom) {
                        handle_clipped_mesh_entry(dlist_interp, entry.unwrap_throw());
                    }
                }
            };
        }
    }
}
