use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::gbi::DisplayList;
use oot_explorer_core::header::{MeshHeader, RoomHeader, SceneHeader};
use oot_explorer_core::mesh::MeshVariant;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx, SegmentResolveError};
use oot_explorer_core::versions;
use scoped_owner::ScopedOwner;

fn main() {
    let rom_data =
        std::fs::read("Legend of Zelda, The - Ocarina of Time (U) (V1.0) [!].z64").unwrap();
    let rom = Rom::new(&rom_data);

    ScopedOwner::with_scope(|scope| {
        let mut fs = LazyFileSystem::new(rom, versions::oot_ntsc_10::FILE_TABLE_ROM_ADDR);

        for (i, entry) in versions::oot_ntsc_10::get_scene_table(scope, &mut fs)
            .iter()
            .enumerate()
        {
            let scene = entry.scene(scope, &mut fs);
            examine_scene(scope, &mut fs, i, scene);
        }
    });
}

fn examine_scene<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    scene_index: usize,
    scene: Scene<'a>,
) {
    let ctx = {
        let mut ctx = SegmentCtx::new();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx
    };
    for header in scene.headers() {
        match header {
            SceneHeader::RoomList(header) => {
                for (room_index, room_list_entry) in header.room_list(&ctx).iter().enumerate() {
                    let room = room_list_entry.room(scope, fs);
                    let ctx = {
                        let mut ctx = ctx.clone();
                        ctx.set(Segment::ROOM, room.data(), room.vrom_range());
                        ctx
                    };
                    for header in room.headers() {
                        match header {
                            RoomHeader::Mesh(header) => {
                                enumerate_meshes(&ctx, scene_index, room_index, header, |dlist| {
                                    dlist.parse(|_| ())
                                });
                            }
                            _ => (),
                        }
                    }
                }
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
