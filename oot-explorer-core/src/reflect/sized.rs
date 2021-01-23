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

pub const fn place_field(offset: usize, align_bits: u32) -> usize {
    let one_less_than_align = (1 << align_bits) - 1;
    let align_mask = !one_less_than_align;
    (offset + one_less_than_align) & align_mask
}

pub const fn max_align(x: u32, y: u32) -> u32 {
    if x > y {
        x
    } else {
        y
    }
}

#[cfg(test)]
mod tests {
    use super::place_field;

    #[test]
    fn place_field_test() {
        assert_eq!(place_field(0, 0), 0);
        assert_eq!(place_field(1, 0), 1);
        assert_eq!(place_field(2, 0), 2);
        assert_eq!(place_field(3, 0), 3);

        assert_eq!(place_field(0, 1), 0);
        assert_eq!(place_field(1, 1), 2);
        assert_eq!(place_field(2, 1), 2);
        assert_eq!(place_field(3, 1), 4);
        assert_eq!(place_field(4, 1), 4);
        assert_eq!(place_field(5, 1), 6);

        assert_eq!(place_field(0, 2), 0);
        assert_eq!(place_field(1, 2), 4);
        assert_eq!(place_field(2, 2), 4);
        assert_eq!(place_field(3, 2), 4);
        assert_eq!(place_field(4, 2), 4);
        assert_eq!(place_field(5, 2), 8);
        assert_eq!(place_field(6, 2), 8);
        assert_eq!(place_field(7, 2), 8);
        assert_eq!(place_field(8, 2), 8);
        assert_eq!(place_field(9, 2), 12);
    }
}
