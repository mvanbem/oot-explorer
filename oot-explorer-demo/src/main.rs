use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_core::header::SceneHeader;
use oot_explorer_core::rom::Rom;
use oot_explorer_core::scene::Scene;
use oot_explorer_core::segment::{Segment, SegmentCtx};
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
            println!("scene #{}", i);
            let scene = entry.scene(scope, &mut fs);
            examine_scene(scope, &mut fs, scene);
        }
    });
}

fn examine_scene<'a>(scope: &'a ScopedOwner, fs: &mut LazyFileSystem<'a>, scene: Scene<'a>) {
    let ctx = {
        let mut ctx = SegmentCtx::new();
        ctx.set(Segment::SCENE, scene.data(), scene.vrom_range());
        ctx
    };
    for header in scene.headers() {
        println!("  scene header: {:?}", header);
        match header {
            SceneHeader::RoomList(header) => {
                for room_list_entry in header.room_list(&ctx) {
                    println!("    room at {:?}", room_list_entry.start());
                    let room = room_list_entry.room(scope, fs);
                    for header in room.headers() {
                        println!("      room header: {:?}", header);
                    }
                }
            }
            _ => (),
        }
    }
}
