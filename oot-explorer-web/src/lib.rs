use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::header::{RoomHeader, SceneHeader};
use oot_explorer_core::mesh::MeshVariant;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::room::Room;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx};
use oot_explorer_core::versions;
use scoped_owner::ScopedOwner;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod expr;
mod gl;

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

#[wasm_bindgen(js_name = processScene)]
pub fn process_scene(rom_data: Box<[u8]>, scene_index: usize) -> js_sys::Array {
    let rom = Rom::new(&rom_data);
    ScopedOwner::with_scope(|scope| {
        let mut fs = LazyFileSystem::new(rom, versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR);
        let mut dlist_interp = gl::DisplayListInterpreter::new();

        let scene = versions::oot_ntsc_10::get_scene_table(scope, &mut fs)
            .get(scene_index)
            .scene(scope, &mut fs);
        examine_scene(scope, &mut fs, scene, &mut dlist_interp);

        println!("total_dlists: {}", dlist_interp.total_dlists());
        println!("total_instructions: {}", dlist_interp.total_instructions());
        println!("unmapped_calls: {:?}", dlist_interp.unmapped_calls());
        println!("max_depth: {}", dlist_interp.max_depth());
        println!("total_lit_verts: {}", dlist_interp.total_lit_verts());
        println!("total_unlit_verts: {}", dlist_interp.total_unlit_verts());

        let js_result = js_sys::Array::new();
        dlist_interp.for_each_batch(|batch| {
            let js_batch = js_sys::Object::new();

            js_sys::Reflect::set(
                &js_batch,
                &"fragmentShader".into(),
                &batch.fragment_shader().into(),
            )
            .unwrap_throw();

            js_sys::Reflect::set(
                &js_batch,
                &"vertexData".into(),
                &new_array_buffer(batch.vertex_data()),
            )
            .unwrap_throw();
            js_result.push(&js_batch);
        });

        js_result
    })
}

pub fn process_all_scenes(rom_data: &[u8]) {
    let rom = Rom::new(rom_data);
    ScopedOwner::with_scope(|scope| {
        let mut fs = LazyFileSystem::new(rom, versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR);
        let mut dlist_interp = gl::DisplayListInterpreter::new();

        for entry in versions::oot_ntsc_10::get_scene_table(scope, &mut fs) {
            let scene = entry.scene(scope, &mut fs);
            examine_scene(scope, &mut fs, scene, &mut dlist_interp);
            dlist_interp.clear_batches();
        }
    })
}

fn examine_scene<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    scene: Scene<'a>,
    dlist_interp: &mut gl::DisplayListInterpreter,
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
    dlist_interp: &mut gl::DisplayListInterpreter,
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
                        if let Some(dlist) = entry.opaque_display_list(&cpu_ctx) {
                            dlist_interp.interpret(&rsp_ctx, dlist);
                        }
                        if let Some(dlist) = entry.translucent_display_list(&cpu_ctx) {
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
