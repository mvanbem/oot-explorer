use crate::header::scene::{SceneHeader, SCENE_HEADER_DESC};
use crate::reflect::primitive::{I8_DESC, U16_DESC, U8_DESC};

compile_interfaces! {
    struct Scene {
        struct SceneHeader[..] headers @0;
    }

    #[size(0x16)]
    struct Lighting {
        // TODO: Support inline struct fields.
        u8 ambient_color_r @0;
        u8 ambient_color_g @1;
        u8 ambient_color_b @2;
        // TODO: Support inline struct fields.
        u8 diffuse_color_a_r @3;
        u8 diffuse_color_a_g @4;
        u8 diffuse_color_a_b @5;
        // TODO: Support arrays of fixed size.
        i8 diffuse_direction_a_x @6;
        i8 diffuse_direction_a_y @7;
        i8 diffuse_direction_a_z @8;
        // TODO: Support inline struct fields.
        u8 diffuse_color_b_r @9;
        u8 diffuse_color_b_g @0xa;
        u8 diffuse_color_b_b @0xb;
        // TODO: Support arrays of fixed size.
        i8 diffuse_direction_b_x @0xc;
        i8 diffuse_direction_b_y @0xd;
        i8 diffuse_direction_b_z @0xe;
        // TODO: Support inline struct fields.
        u8 fog_color_r @0xf;
        u8 fog_color_g @0x10;
        u8 fog_color_b @0x11;

        // TODO: This is a bitfield!
        // fog_start = fog_start_and_flags & 0x03ff
        // flags = fog_start_and_flags >> 10
        u16 fog_start_and_flags @0x12;
        u16 draw_distance @0x14;
    }
}
