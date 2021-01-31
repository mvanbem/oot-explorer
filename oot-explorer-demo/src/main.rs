use oot_explorer_game_data::gbi::DisplayList;
use oot_explorer_game_data::header_room::{MeshHeader, RoomHeaderVariant};
use oot_explorer_game_data::header_scene::SceneHeaderVariant;
use oot_explorer_game_data::mesh::{Background, JfifMeshVariant, MeshEntry, MeshVariant};
use oot_explorer_game_data::room::{Room, ROOM_DESC};
use oot_explorer_game_data::scene::{Scene, SCENE_DESC};
use oot_explorer_game_data::versions::oot_ntsc_10;
use oot_explorer_gl::display_list_interpreter::{DisplayListInterpreter, DisplayListOpacity};
use oot_explorer_gl::shader_state::TextureDescriptor;
use oot_explorer_read::VromProxy;
use oot_explorer_rom::Rom;
use oot_explorer_segment::{Segment, SegmentTable};
use oot_explorer_vrom::{decompress, FileIndex, FileTable, OwnedVrom, Vrom};
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

mod reflect_text;

struct Context {
    file_table: FileTable,
    vrom: OwnedVrom,
}

fn main() {
    // Load and decompress the game data. Put the results in an Arc to share with worker threads.
    let (file_table, vrom) = decompress(
        Rom(&std::fs::read("Legend of Zelda, The - Ocarina of Time (U) (V1.0) [!].z64").unwrap()),
        oot_ntsc_10::FILE_TABLE_ROM_ADDR,
    )
    .unwrap();
    let ctx = Arc::new(Context { file_table, vrom });

    // A channel for the main thread to send work to the worker threads.
    let (sender, receiver) = crossbeam::channel::bounded(0);

    // Spawn worker threads to dump textures.
    let mut join_handles = vec![];
    for _ in 0..8 {
        let ctx = Arc::clone(&ctx);
        let receiver = receiver.clone();
        join_handles.push(std::thread::spawn(move || {
            while let Ok(texture) = receiver.recv() {
                dump_texture(ctx.vrom.borrow(), &texture);
            }
        }));
    }

    // Scan the game data on the main thread.
    let mut dlist_interp = DisplayListInterpreter::new();
    for (scene_index, entry) in oot_ntsc_10::get_scene_table(&ctx.file_table)
        .unwrap()
        .iter(ctx.vrom.borrow())
        .enumerate()
    {
        let scene = entry
            .unwrap()
            .scene(ctx.vrom.borrow())
            .unwrap()
            .into_inner();
        examine_scene(
            &ctx.file_table,
            ctx.vrom.borrow(),
            &SegmentTable::default(),
            &mut dlist_interp,
            scene_index,
            scene,
        );
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
    drop(sender);

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
}

fn examine_scene(
    file_table: &FileTable,
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    dlist_interp: &mut DisplayListInterpreter,
    scene_index: usize,
    scene: Scene,
) {
    let segment_table = segment_table.with(Segment::SCENE, scene.addr()).unwrap();

    reflect_text::dump(vrom, &segment_table, SCENE_DESC, scene.addr(), 0);
    println!();

    for result in scene.headers(vrom) {
        let header = result.unwrap();
        match header.variant(vrom) {
            SceneHeaderVariant::RoomList(header) => {
                for (room_index, room_list_entry) in header
                    .room_list(vrom, &segment_table)
                    .unwrap()
                    .iter(vrom)
                    .enumerate()
                {
                    let room = room_list_entry.unwrap().room(vrom).unwrap().into_inner();
                    examine_room(
                        file_table,
                        vrom,
                        &segment_table,
                        dlist_interp,
                        scene_index,
                        room_index,
                        room,
                    );
                }
            }
            _ => (),
        }
    }
}

fn examine_room(
    file_table: &FileTable,
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    dlist_interp: &mut DisplayListInterpreter,
    scene_index: usize,
    room_index: usize,
    room: Room,
) {
    let segment_table = segment_table.with(Segment::ROOM, room.addr()).unwrap();

    reflect_text::dump(vrom, &segment_table, ROOM_DESC, room.addr(), 0);
    println!();

    for result in room.headers(vrom) {
        let header = result.unwrap();
        match header.variant(vrom) {
            RoomHeaderVariant::Mesh(header) => {
                enumerate_meshes(
                    vrom,
                    &segment_table,
                    scene_index,
                    room_index,
                    header,
                    |translucent, dlist| {
                        dlist_interp
                            .interpret(vrom, &segment_table, translucent, dlist)
                            .unwrap();
                    },
                    |background| {
                        let segment_addr = background.ptr(vrom);
                        let vrom_addr = segment_table.resolve(segment_addr).unwrap();

                        let mut index = FileIndex(0);
                        let end_addr = loop {
                            let range = file_table.file_vrom_range(index).unwrap();
                            if range.contains(&vrom_addr) {
                                break range.end;
                            }
                            index = FileIndex(index.0 + 1);
                        };

                        std::fs::write(
                            {
                                let mut path = PathBuf::from("./backgrounds");
                                path.push(&format!("0x{:08x}.jpg", vrom_addr.0));
                                path
                            },
                            // TODO: Limit this to the JFIF data. As written, all decompressed game
                            // data in the same file after the JFIF data also gets appended to the
                            // exported file. This should be harmless, but is silly.
                            vrom.slice(vrom_addr..end_addr).unwrap(),
                        )
                        .unwrap();
                    },
                );
            }
            _ => (),
        }
    }
}

fn enumerate_meshes<F, G>(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    scene_index: usize,
    room_index: usize,
    header: MeshHeader,
    mut f: F,
    mut g: G,
) where
    F: FnMut(DisplayListOpacity, DisplayList),
    G: FnMut(Background),
{
    match header.mesh(vrom, segment_table).unwrap().variant(vrom) {
        MeshVariant::Simple(mesh) => {
            for result in mesh.entries(vrom, segment_table).unwrap().iter(vrom) {
                enumerate_mesh_entry(
                    vrom,
                    segment_table,
                    scene_index,
                    room_index,
                    result.unwrap(),
                    &mut f,
                );
            }
        }

        MeshVariant::Jfif(jfif) => match jfif.variant(vrom) {
            JfifMeshVariant::Single(single) => {
                enumerate_mesh_entry(
                    vrom,
                    segment_table,
                    scene_index,
                    room_index,
                    single.mesh_entry(vrom, segment_table).unwrap(),
                    &mut f,
                );
                g(single.background(vrom));
            }
            JfifMeshVariant::Multiple(multiple) => {
                enumerate_mesh_entry(
                    vrom,
                    segment_table,
                    scene_index,
                    room_index,
                    multiple.mesh_entry(vrom, segment_table).unwrap(),
                    &mut f,
                );
                for result in multiple
                    .background_entries(vrom, segment_table)
                    .unwrap()
                    .iter(vrom)
                {
                    g(result.unwrap().background(vrom));
                }
            }
        },

        MeshVariant::Clipped(mesh) => {
            for result in mesh.entries(vrom, segment_table).unwrap().iter(vrom) {
                enumerate_mesh_entry(
                    vrom,
                    segment_table,
                    scene_index,
                    room_index,
                    result.unwrap(),
                    &mut f,
                );
            }
        }
    }
}

fn enumerate_mesh_entry<F>(
    vrom: Vrom<'_>,
    segment_table: &SegmentTable,
    scene_index: usize,
    room_index: usize,
    entry: impl MeshEntry + Copy,
    mut f: F,
) where
    F: FnMut(DisplayListOpacity, DisplayList),
{
    match entry.opaque_display_list(vrom, segment_table) {
        Ok(Some(dlist)) => f(DisplayListOpacity::Opaque, dlist),
        Ok(None) => (),
        Err(e) => {
            eprintln!(
                "scene {}, room {}: while resolving display list: {}",
                scene_index, room_index, e,
            )
        }
    }
    match entry.translucent_display_list(vrom, segment_table) {
        Ok(Some(dlist)) => f(DisplayListOpacity::Translucent, dlist),
        Ok(None) => (),
        Err(e) => {
            eprintln!(
                "scene {}, room {}: while resolving display list: {}",
                scene_index, room_index, e,
            )
        }
    }
}

fn dump_texture(vrom: Vrom<'_>, texture: &TextureDescriptor) {
    let src = texture.source.src().unwrap();

    let decoded_texture = match oot_explorer_gl::texture::decode(vrom, texture) {
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
