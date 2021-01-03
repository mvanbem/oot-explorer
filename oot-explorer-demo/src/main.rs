use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::gbi::DisplayList;
use oot_explorer_core::header::{MeshHeader, RoomHeader, SceneHeader};
use oot_explorer_core::mesh::MeshVariant;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::room::Room;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx, SegmentResolveError};
use oot_explorer_core::versions;
use oot_explorer_gl::display_list_interpreter::DisplayListInterpreter;
use oot_explorer_gl::shader_state::TextureDescriptor;
use scoped_owner::ScopedOwner;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    let rom_data = Arc::new(
        std::fs::read("Legend of Zelda, The - Ocarina of Time (U) (V1.0) [!].z64").unwrap(),
    );

    let (sender, receiver) = crossbeam::channel::bounded(0);
    let mut join_handles = vec![];
    for _ in 0..8 {
        let rom_data = Arc::clone(&rom_data);
        let receiver = receiver.clone();
        join_handles.push(std::thread::spawn(move || {
            ScopedOwner::with_scope(|scope| {
                let rom = Rom::new(&rom_data);
                let mut fs = LazyFileSystem::new(
                    rom,
                    oot_explorer_core::versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR,
                );
                while let Ok(texture) = receiver.recv() {
                    dump_texture(scope, &mut fs, &texture);
                }
            });
        }));
    }

    ScopedOwner::with_scope(|scope| {
        let rom = Rom::new(&rom_data);
        let mut fs = LazyFileSystem::new(rom, versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR);
        let mut dlist_interp = DisplayListInterpreter::new();

        let ctx = SegmentCtx::new();
        for (scene_index, entry) in versions::oot_ntsc_10::get_scene_table(scope, &mut fs)
            .iter()
            .enumerate()
        {
            let scene = entry.scene(scope, &mut fs);
            examine_scene(scope, &mut fs, &ctx, &mut dlist_interp, scene_index, scene);
            dlist_interp.clear_batches();
        }
        println!("total_dlists: {}", dlist_interp.total_dlists());
        println!("total_instructions: {}", dlist_interp.total_instructions());
        println!("unmapped_calls: {:?}", dlist_interp.unmapped_calls());
        println!("unmapped_textures: {:?}", dlist_interp.unmapped_textures());
        println!("max_depth: {}", dlist_interp.max_depth());
        println!("total_lit_verts: {}", dlist_interp.total_lit_verts());
        println!("total_unlit_verts: {}", dlist_interp.total_unlit_verts());
        println!("unique_textures: {:#?}", dlist_interp.unique_textures());

        for texture in dlist_interp.iter_textures() {
            sender.send(texture.clone()).unwrap();
        }
    });
    drop(sender);

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
}

fn examine_scene<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    ctx: &SegmentCtx<'a>,
    dlist_interp: &mut DisplayListInterpreter,
    scene_index: usize,
    scene: Scene<'a>,
) {
    let ctx = {
        let mut ctx = ctx.clone();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx
    };
    for header in scene.headers() {
        match header {
            SceneHeader::RoomList(header) => {
                for (room_index, room_list_entry) in header.room_list(&ctx).iter().enumerate() {
                    examine_room(
                        &ctx,
                        dlist_interp,
                        scene_index,
                        room_index,
                        room_list_entry.room(scope, fs),
                    );
                }
            }
            _ => (),
        }
    }
}

fn examine_room<'a>(
    ctx: &SegmentCtx<'a>,
    dlist_interp: &mut DisplayListInterpreter,
    scene_index: usize,
    room_index: usize,
    room: Room<'a>,
) {
    let ctx = {
        let mut ctx = ctx.clone();
        ctx.set(Segment::ROOM, room.data(), room.vrom_range());
        ctx
    };
    for header in room.headers() {
        match header {
            RoomHeader::Mesh(header) => {
                enumerate_meshes(&ctx, scene_index, room_index, header, |dlist| {
                    dlist_interp.interpret(&ctx, dlist);
                });
            }
            _ => (),
        }
    }
}

fn enumerate_meshes<'a, F>(
    ctx: &SegmentCtx<'a>,
    scene_index: usize,
    room_index: usize,
    header: MeshHeader<'a>,
    mut f: F,
) where
    F: FnMut(DisplayList),
{
    match header.mesh(&ctx).variant() {
        MeshVariant::Simple(mesh) => {
            for entry in mesh.entries(&ctx) {
                match entry.opaque_display_list(ctx) {
                    Ok(Some(dlist)) => f(dlist),
                    Ok(None) => (),
                    Err(SegmentResolveError::Unmapped { .. }) => {
                        eprintln!(
                            "scene {}, room {}: unmapped segment for display list at {:?}",
                            scene_index,
                            room_index,
                            entry.opaque_display_list_ptr().unwrap(),
                        )
                    }
                }
                match entry.translucent_display_list(ctx) {
                    Ok(Some(dlist)) => f(dlist),
                    Ok(None) => (),
                    Err(SegmentResolveError::Unmapped { .. }) => {
                        eprintln!(
                            "scene {}, room {}: unmapped segment for display list at {:?}",
                            scene_index,
                            room_index,
                            entry.translucent_display_list_ptr().unwrap(),
                        )
                    }
                }
            }
        }
        MeshVariant::Jfif(_) => (),
        MeshVariant::Clipped(mesh) => {
            for entry in mesh.entries(&ctx) {
                match entry.opaque_display_list(ctx) {
                    Ok(Some(dlist)) => f(dlist),
                    Ok(None) => (),
                    Err(SegmentResolveError::Unmapped { .. }) => {
                        eprintln!(
                            "scene {}, room {}: unmapped segment for display list at {:?}",
                            scene_index,
                            room_index,
                            entry.opaque_display_list_ptr().unwrap(),
                        )
                    }
                }
                match entry.translucent_display_list(ctx) {
                    Ok(Some(dlist)) => f(dlist),
                    Ok(None) => (),
                    Err(SegmentResolveError::Unmapped { .. }) => {
                        eprintln!(
                            "scene {}, room {}: unmapped segment for display list at {:?}",
                            scene_index,
                            room_index,
                            entry.translucent_display_list_ptr().unwrap(),
                        )
                    }
                }
            }
        }
    }
}

fn dump_texture<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    texture: &TextureDescriptor,
) {
    let src = texture.source.src().unwrap();

    let decoded_texture = match oot_explorer_gl::texture::decode(scope, fs, texture) {
        Ok(decoded_texture) => decoded_texture,
        Err(e) => {
            eprintln!("WARNING: failed to decode texture {:?}: {}", src, e);
            return;
        }
    };

    let mut file = {
        let mut path = PathBuf::from("./textures");
        let hash = {
            let mut hasher = DefaultHasher::new();
            texture.hash(&mut hasher);
            hasher.finish()
        };
        path.push(format!("0x{:08x}_0x{:016x}.png", src.0, hash));
        BufWriter::new(File::create(path).unwrap())
    };

    let mut encoder = png::Encoder::new(
        &mut file,
        decoded_texture.width.try_into().unwrap(),
        decoded_texture.height.try_into().unwrap(),
    );
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&decoded_texture.data).unwrap();
    drop(writer);
    file.flush().unwrap()
}
