use crate::header_room::{RoomHeader, ROOM_HEADER_DESC};

compile_interfaces! {
    //  TODO: Don't specify a size for unsized types!
    #[layout(size = 8, align_bits = 2)]
    struct Room {
        struct RoomHeader[..] headers @0;
    }
}
