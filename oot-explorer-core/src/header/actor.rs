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
}
