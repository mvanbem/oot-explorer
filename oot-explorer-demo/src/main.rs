use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::gbi::DisplayList;
use oot_explorer_core::header::room::variant::mesh::MeshHeader;
use oot_explorer_core::header::room::variant::RoomHeaderVariant;
use oot_explorer_core::header::scene::variant::SceneHeaderVariant;
use oot_explorer_core::mesh::{Background, JfifMeshVariant, MeshVariant, SimpleMeshEntry};
use oot_explorer_core::reflect::DebugReflect;
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
        println!("unmapped_matrices: {:?}", dlist_interp.unmapped_matrices());
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

    println!("{:#?}", DebugReflect(&scene));

    for header in scene.headers() {
        match header {
            SceneHeaderVariant::RoomList(header) => {
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
            RoomHeaderVariant::Mesh(header) => {
                enumerate_meshes(
                    &ctx,
                    scene_index,
                    room_index,
                    header,
                    |translucent, dlist| {
                        dlist_interp.interpret(&ctx, translucent, dlist);
                    },
                    |background| {
                        let segment_ptr = background.ptr();
                        let vrom_addr = ctx.resolve_vrom(segment_ptr).unwrap().start;

                        std::fs::write(
                            {
                                let mut path = PathBuf::from("./backgrounds");
                                path.push(&format!("0x{:08x}.jpg", vrom_addr.0));
                                path
                            },
                            // TODO: Limit this to the JFIF data. As written, all data in the
                            // containing file after the JFIF data also gets appended to the
                            // exported file. This should be harmless, but wastes space and is
                            // sloppy.
                            ctx.resolve(segment_ptr).unwrap(),
                        )
                        .unwrap();
                    },
                );
            }
            _ => (),
        }
    }
}

fn enumerate_meshes<'a, F, G>(
    ctx: &SegmentCtx<'a>,
    scene_index: usize,
    room_index: usize,
    header: MeshHeader<'a>,
    mut f: F,
    mut g: G,
) where
    F: FnMut(bool, DisplayList),
    G: FnMut(Background<'a>),
{
    let mut handle_simple_mesh_entry = |entry: SimpleMeshEntry<'a>| {
        match entry.opaque_display_list(ctx) {
            Ok(Some(dlist)) => f(false, dlist),
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
            Ok(Some(dlist)) => f(true, dlist),
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
    };

    match header.mesh(&ctx).variant() {
        MeshVariant::Simple(mesh) => {
            mesh.entries(&ctx).iter().for_each(handle_simple_mesh_entry);
        }
        MeshVariant::Jfif(jfif) => match jfif.variant() {
            JfifMeshVariant::Single(single) => {
                handle_simple_mesh_entry(single.mesh_entry(ctx));
                g(single.background());
            }
            JfifMeshVariant::Multiple(multiple) => {
                multiple
                    .mesh_entries(ctx)
                    .iter()
                    .for_each(handle_simple_mesh_entry);
                for entry in multiple.background_entries(ctx) {
                    g(entry.background());
                }
            }
        },
        MeshVariant::Clipped(mesh) => {
            for entry in mesh.entries(ctx) {
                match entry.opaque_display_list(ctx) {
                    Ok(Some(dlist)) => f(false, dlist),
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
                    Ok(Some(dlist)) => f(true, dlist),
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
