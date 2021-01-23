use crate::header::room::{RoomHeader, ROOM_HEADER_DESC};

compile_interfaces! {
    struct Room {
        struct RoomHeader[..] headers @0;
    }
}
