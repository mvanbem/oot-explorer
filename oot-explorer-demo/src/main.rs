use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::gbi::{DisplayList, Qu1_11, TextureDepth, TextureFormat};
use oot_explorer_core::header::{MeshHeader, RoomHeader, SceneHeader};
use oot_explorer_core::mesh::MeshVariant;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::room::Room;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx, SegmentResolveError};
use oot_explorer_core::versions;
use oot_explorer_gl::display_list_interpreter::DisplayListInterpreter;
use oot_explorer_gl::rcp::TmemSource;
use oot_explorer_gl::shader_state::TextureDescriptor;
use scoped_owner::ScopedOwner;
use std::collections::hash_map::DefaultHasher;
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

// Applies both layers of TMEM word swapping. One is performed by the LoadBlock command based on
// load_dxt and the word offset. The other is performed by the RDP based on y. These may cancel out.
fn word_swap(offset: usize, load_dxt: Qu1_11, render_y: usize) -> usize {
    let load_line = ((offset / 8) * (load_dxt.0 as usize)) >> 11;
    let load_swap = load_line & 1 == 1;

    let render_swap = render_y & 1 == 1;

    if load_swap != render_swap {
        offset ^ 0x4
    } else {
        offset
    }
}

fn expand_5_to_8(x: u8) -> u8 {
    (x << 3) | (x >> 2)
}

fn dump_texture<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    texture: &TextureDescriptor,
) {
    if let TmemSource::LoadBlock {
        src_ptr,
        src_format,
        src_depth,
        load_dxt,
        load_format,
        load_depth,
        ..
    } = texture.source
    {
        // Format conversion during load is not implemented.
        assert_eq!(src_format, load_format);
        assert_eq!(src_depth, load_depth);

        let src = fs.get_virtual_slice(
            scope,
            src_ptr
                ..src_ptr
                    + (8 * texture.render_width
                        / texture.render_depth.texels_per_tmem_word::<usize>()
                        + 8 * (texture.render_height - 1) * texture.render_stride)
                        as u32,
        );
        let stride_bytes = 8 * texture.render_stride;

        let mut dst = Vec::with_capacity(4 * texture.render_width * texture.render_height);
        match (texture.render_depth, texture.render_format) {
            (TextureDepth::Bits4, TextureFormat::Ia) => {
                for y in 0..texture.render_height {
                    for x in (0..texture.render_width).step_by(2) {
                        let offset = word_swap(stride_bytes * y + x / 2, load_dxt, y);
                        let x = src[offset];
                        let i1 = (x & 0xe0) | ((x >> 3) & 0x8c) | ((x >> 6) & 0x03);
                        let a1 = if x & 0x10 == 0x10 { 0xff } else { 0x00 };
                        let i2 = ((x << 4) & 0xe0) | ((x << 1) & 0x8c) | ((x >> 2) & 0x03);
                        let a2 = if x & 0x01 == 0x01 { 0xff } else { 0x00 };
                        dst.extend_from_slice(&[i1, i1, i1, a1, i2, i2, i2, a2]);
                    }
                }
            }
            (TextureDepth::Bits4, TextureFormat::I) => {
                for y in 0..texture.render_height {
                    for x in (0..texture.render_width).step_by(2) {
                        let offset = word_swap(stride_bytes * y + x / 2, load_dxt, y);
                        let x = src[offset];
                        let i1 = (x & 0xf0) | ((x >> 4) & 0x0f);
                        let i2 = ((x << 4) & 0xf0) | (x & 0x0f);
                        dst.extend_from_slice(&[i1, i1, i1, 255, i2, i2, i2, 255]);
                    }
                }
            }
            (TextureDepth::Bits8, TextureFormat::Ia) => {
                for y in 0..texture.render_height {
                    for x in 0..texture.render_width {
                        let offset = word_swap(stride_bytes * y + x, load_dxt, y);
                        let x = src[offset];
                        let i = (x & 0xf0) | ((x >> 4) & 0x0f);
                        let a = ((x << 4) & 0xf0) | (x & 0x0f);
                        dst.extend_from_slice(&[i, i, i, a]);
                    }
                }
            }
            (TextureDepth::Bits8, TextureFormat::I) => {
                for y in 0..texture.render_height {
                    for x in 0..texture.render_width {
                        let offset = word_swap(stride_bytes * y + x, load_dxt, y);
                        let i = src[offset];
                        dst.extend_from_slice(&[i, i, i, 255]);
                    }
                }
            }
            (TextureDepth::Bits16, TextureFormat::Rgba) => {
                for y in 0..texture.render_height {
                    for x in 0..texture.render_width {
                        let offset = word_swap(stride_bytes * y + 2 * x, load_dxt, y);
                        let x = ((src[offset] as u16) << 8) | src[offset + 1] as u16;
                        let r = expand_5_to_8(((x >> 11) & 0x1f) as u8);
                        let g = expand_5_to_8(((x >> 6) & 0x1f) as u8);
                        let b = expand_5_to_8(((x >> 1) & 0x1f) as u8);
                        let a = if x & 0x01 == 0x01 { 0xff } else { 0x00 };
                        dst.extend_from_slice(&[r, g, b, a]);
                    }
                }
            }
            (TextureDepth::Bits16, TextureFormat::Ia) => {
                for y in 0..texture.render_height {
                    for x in 0..texture.render_width {
                        let offset = word_swap(stride_bytes * y + 2 * x, load_dxt, y);
                        let i = src[offset];
                        let a = src[offset + 1];
                        dst.extend_from_slice(&[i, i, i, a]);
                    }
                }
            }
            x => {
                eprintln!("unimplemented format: {:?}", x);
                return;
            }
        }

        let mut file = {
            let mut path = PathBuf::from("./textures");
            let hash = {
                let mut hasher = DefaultHasher::new();
                texture.hash(&mut hasher);
                hasher.finish()
            };
            path.push(format!("0x{:08x}_0x{:016x}.png", src_ptr.0, hash));
            BufWriter::new(File::create(path).unwrap())
        };

        let mut encoder = png::Encoder::new(
            &mut file,
            texture.render_width as u32,
            texture.render_height as u32,
        );
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&dst).unwrap();
        drop(writer);
        file.flush().unwrap()
    }
}
