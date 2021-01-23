pub trait ReflectSized {
    const SIZE: usize;
    const ALIGN_BITS: u32 = Self::SIZE.trailing_zeros();
}

impl ReflectSized for bool {
    const SIZE: usize = 1;
    const ALIGN_BITS: u32 = 0;
}

impl ReflectSized for u8 {
    const SIZE: usize = 1;
    const ALIGN_BITS: u32 = 0;
}

impl ReflectSized for i8 {
    const SIZE: usize = 1;
    const ALIGN_BITS: u32 = 0;
}

impl ReflectSized for u16 {
    const SIZE: usize = 2;
    const ALIGN_BITS: u32 = 1;
}

impl ReflectSized for i16 {
    const SIZE: usize = 2;
    const ALIGN_BITS: u32 = 1;
}

impl ReflectSized for u32 {
    const SIZE: usize = 4;
    const ALIGN_BITS: u32 = 2;
}

impl ReflectSized for i32 {
    const SIZE: usize = 4;
    const ALIGN_BITS: u32 = 2;
}
