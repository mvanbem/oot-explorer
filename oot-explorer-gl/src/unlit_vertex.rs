use byteorder::{NativeEndian, WriteBytesExt};
use oot_explorer_core::gbi;
use std::io::Write;

use crate::FLAGS_UNLIT;

pub trait UnlitVertex {
    fn position(&self) -> [i16; 3];
    fn texcoord(&self) -> [i16; 2];
    fn color(&self) -> [u8; 4];
}
impl<'a> UnlitVertex for gbi::UnlitVertex<'a> {
    fn position(&self) -> [i16; 3] {
        gbi::UnlitVertex::position(*self)
    }
    fn texcoord(&self) -> [i16; 2] {
        gbi::UnlitVertex::texcoord(*self)
    }
    fn color(&self) -> [u8; 4] {
        gbi::UnlitVertex::color(*self)
    }
}

pub fn write_unlit_vertex<T>(dst: &mut [u8; 20], vertex: &T)
where
    T: UnlitVertex,
{
    let mut w = &mut dst[..];
    // [0..=5] Position
    let pos = vertex.position();
    w.write_i16::<NativeEndian>(pos[0]).unwrap();
    w.write_i16::<NativeEndian>(pos[1]).unwrap();
    w.write_i16::<NativeEndian>(pos[2]).unwrap();
    // [6..=7] Padding
    w.write_u16::<NativeEndian>(0).unwrap();
    // [8..=10] Normal (unused for unlit geometry)
    w.write_i8(0).unwrap();
    w.write_i8(0).unwrap();
    w.write_i8(0).unwrap();
    // [11] Flags
    w.write_u8(FLAGS_UNLIT).unwrap();
    // [12..=15] Texture coordinates
    let texcoord = vertex.texcoord();
    w.write_i16::<NativeEndian>(texcoord[0]).unwrap();
    w.write_i16::<NativeEndian>(texcoord[1]).unwrap();
    // [16..=19] Color
    let color = vertex.color();
    w.write_all(&color[..]).unwrap();
    assert_eq!(w.len(), 0);
}
