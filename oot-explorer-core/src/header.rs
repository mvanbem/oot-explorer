pub mod room;
pub mod scene;

use crate::reflect::primitive::U32_DESC;
use crate::reflect::primitive::{I16_DESC, U16_DESC};

compile_interfaces! {
    #[size(0x10)]
    struct Actor {
        u16 actor_number @ 0;
        i16 pos_x @ 2;
        i16 pos_y @ 4;
        i16 pos_z @ 6;
        i16 angle_x @ 8;
        i16 angle_y @ 0xa;
        i16 angle_z @ 0xc;
        u16 init @ 0xe;
    }

    struct AlternateHeadersHeader {
        // TODO: Type this.
        u32 ptr @4;
    }
}
