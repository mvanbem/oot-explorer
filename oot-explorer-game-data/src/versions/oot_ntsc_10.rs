use std::ops::Range;

use crate::scene::Scene;
use oot_explorer_read::{ReadError, Slice};
use oot_explorer_reflect::{RangeSourced, U8_DESC, VROM_ADDR_DESC};
use oot_explorer_rom::RomAddr;
use oot_explorer_vrom::{FileIndex, FileTable, Vrom, VromAddr};

pub const FILE_TABLE_ROM_ADDR: RomAddr = RomAddr(0x00007430);

pub const SCENE_TABLE_FILE_INDEX: FileIndex = FileIndex(0x1b);
pub const SCENE_TABLE_OFFSET: u32 = 0xea440;
pub const SCENE_TABLE_COUNT: u32 = 101;

pub fn get_scene_table(file_table: &FileTable) -> Result<Slice<SceneTableEntry>, ReadError> {
    Ok(Slice::new(
        file_table.file_vrom_range(SCENE_TABLE_FILE_INDEX)?.start + SCENE_TABLE_OFFSET,
        SCENE_TABLE_COUNT,
    ))
}

compile_interfaces! {
    #[layout(size = 0x14, align_bits = 2)]
    struct SceneTableEntry {
        VromAddr scene_start @0;
        VromAddr scene_end @4;
        VromAddr raw_title_card_start @8;
        VromAddr raw_title_card_end @0xc;
        u8 unknown_a @0x10;
        u8 render_init_function @0x11;
        u8 unknown_b @0x12;
    }
}

impl SceneTableEntry {
    pub fn scene_range(self, vrom: Vrom<'_>) -> Range<VromAddr> {
        self.scene_start(vrom)..self.scene_end(vrom)
    }

    pub fn scene(self, vrom: Vrom<'_>) -> Result<RangeSourced<Scene>, ReadError> {
        RangeSourced::from_vrom_range(vrom, self.scene_range(vrom))
    }

    pub fn title_card_range(self, vrom: Vrom<'_>) -> Option<Range<VromAddr>> {
        let start = self.raw_title_card_start(vrom);
        if start == VromAddr(0) {
            None
        } else {
            Some(start..self.raw_title_card_end(vrom))
        }
    }
}
