use oot_explorer_game_data::header_scene::SceneHeaderVariant;
use oot_explorer_game_data::room::ROOM_DESC;
use oot_explorer_game_data::scene::SCENE_DESC;
use oot_explorer_game_data::versions;
use oot_explorer_read::VromProxy;
use oot_explorer_reflect::{StructFieldLocation, TypeDescriptor};
use oot_explorer_segment::{Segment, SegmentTable};
use oot_explorer_vrom::VromAddr;
use std::ops::Range;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::UnwrapThrowExt;

use crate::reflect_value::{reflect_field, ReflectResult};
use crate::Context;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ReflectRoot {
    #[wasm_bindgen(skip)]
    pub vrom_range: Range<VromAddr>,

    #[wasm_bindgen(skip)]
    pub segment_table: SegmentTable,

    #[wasm_bindgen(skip)]
    pub desc: TypeDescriptor,

    #[wasm_bindgen(skip)]
    pub description: String,
}

#[wasm_bindgen]
impl ReflectRoot {
    #[wasm_bindgen(js_name = forScene)]
    pub fn for_scene(ctx: &Context, scene_index: usize) -> Self {
        let inner = ctx.inner.lock().unwrap_throw();
        let file_table = inner.file_table.as_ref().unwrap_throw();
        let vrom = inner.vrom.as_ref().unwrap_throw().borrow();

        let vrom_range = versions::oot_ntsc_10::get_scene_table(file_table)
            .unwrap_throw()
            .iter(vrom)
            .nth(scene_index)
            .unwrap_throw()
            .unwrap_throw()
            .scene_range(vrom);
        let segment_table = SegmentTable::new().with(Segment::SCENE, vrom_range.start);

        Self {
            vrom_range,
            segment_table,
            desc: SCENE_DESC,
            description: format!("Scene {}", scene_index),
        }
    }

    #[wasm_bindgen(js_name = forRoom)]
    pub fn for_room(ctx: &Context, scene_index: usize, room_index: usize) -> Self {
        let inner = ctx.inner.lock().unwrap_throw();
        let file_table = inner.file_table.as_ref().unwrap_throw();
        let vrom = inner.vrom.as_ref().unwrap_throw().borrow();

        let scene_table_entry = versions::oot_ntsc_10::get_scene_table(file_table)
            .unwrap_throw()
            .iter(vrom)
            .nth(scene_index)
            .unwrap_throw()
            .unwrap_throw();
        let scene = scene_table_entry.scene(vrom).unwrap_throw().into_inner();
        let segment_table = SegmentTable::new().with(Segment::SCENE, scene.addr());

        let room_list_entry = scene
            .headers(vrom)
            .find_map(|header| match header.unwrap_throw().variant(vrom) {
                SceneHeaderVariant::RoomList(room_list) => Some(room_list),
                _ => None,
            })
            .unwrap_throw()
            .room_list(vrom, &segment_table)
            .unwrap_throw()
            .iter(vrom)
            .nth(room_index)
            .unwrap_throw()
            .unwrap_throw();
        let vrom_range = room_list_entry.room_range(vrom);
        let segment_table = segment_table.with(Segment::ROOM, vrom_range.start);

        Self {
            vrom_range,
            segment_table,
            desc: ROOM_DESC,
            description: format!("Room {} in Scene {}", room_index, scene_index),
        }
    }

    #[wasm_bindgen]
    pub fn reflect(&self, ctx: &Context) -> ReflectResult {
        let ctx_ref = ctx.inner.lock().unwrap_throw();
        let vrom = ctx_ref.vrom.as_ref().unwrap_throw().borrow();

        reflect_field(
            vrom,
            &self.segment_table,
            self.vrom_range.start,
            None,
            &StructFieldLocation::Simple { offset: 0 },
            self.desc,
        )
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }
}
