pub mod oot_ntsc_10 {
    use crate::fs::{LazyFileSystem, VromAddr};
    use crate::rom::RomAddr;
    use crate::scene::Scene;
    use crate::slice::{Slice, StructReader};
    use byteorder::{BigEndian, ReadBytesExt};
    use scoped_owner::ScopedOwner;
    use std::fmt::{self, Debug, Formatter};
    use std::ops::Range;

    pub const FILE_TABLE_ROM_ADDR: RomAddr = RomAddr(0x00007430);

    pub const SCENE_TABLE_FILE_INDEX: usize = 0x1b;
    pub const SCENE_TABLE_OFFSET: usize = 0xea440;
    pub const SCENE_TABLE_COUNT: usize = 101;

    pub fn get_scene_table<'a>(
        scope: &'a ScopedOwner,
        fs: &mut LazyFileSystem<'a>,
    ) -> Slice<'a, SceneTableEntry<'a>> {
        Slice::new(
            &fs.get_file(scope, SCENE_TABLE_FILE_INDEX)[SCENE_TABLE_OFFSET..],
            SCENE_TABLE_COUNT,
        )
    }

    #[derive(Clone, Copy)]
    pub struct SceneTableEntry<'a> {
        data: &'a [u8],
    }
    impl<'a> Debug for SceneTableEntry<'a> {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            f.debug_struct("SceneTableEntry")
                .field("scene_range", &self.scene_range())
                .field("title_card_range", &self.title_card_range())
                .field("unknown_a", &self.unknown_a())
                .field("render_init_function", &self.render_init_function())
                .field("unknown_b", &self.unknown_b())
                .finish()
        }
    }
    impl<'a> StructReader<'a> for SceneTableEntry<'a> {
        const SIZE: usize = 0x14;
        fn new(data: &'a [u8]) -> SceneTableEntry<'a> {
            SceneTableEntry { data }
        }
    }
    impl<'a> SceneTableEntry<'a> {
        pub fn scene_start(self) -> VromAddr {
            VromAddr((&self.data[0x00..]).read_u32::<BigEndian>().unwrap())
        }
        pub fn scene_end(self) -> VromAddr {
            VromAddr((&self.data[0x04..]).read_u32::<BigEndian>().unwrap())
        }
        pub fn raw_title_card_start(self) -> VromAddr {
            VromAddr((&self.data[0x08..]).read_u32::<BigEndian>().unwrap())
        }
        pub fn raw_title_card_end(self) -> VromAddr {
            VromAddr((&self.data[0x0c..]).read_u32::<BigEndian>().unwrap())
        }
        pub fn unknown_a(self) -> u8 {
            self.data[0x10]
        }
        pub fn render_init_function(self) -> u8 {
            self.data[0x11]
        }
        pub fn unknown_b(self) -> u8 {
            self.data[0x12]
        }

        pub fn scene_range(self) -> Range<VromAddr> {
            self.scene_start()..self.scene_end()
        }
        pub fn scene(self, scope: &'a ScopedOwner, fs: &mut LazyFileSystem<'a>) -> Scene<'a> {
            Scene::new(
                self.scene_start(),
                fs.get_virtual_slice_or_die(scope, self.scene_range()),
            )
        }
        pub fn title_card_range(self) -> Option<Range<VromAddr>> {
            let start = self.raw_title_card_start();
            if start == VromAddr(0) {
                None
            } else {
                Some(start..self.raw_title_card_end())
            }
        }
    }
}
