use byteorder::{NativeEndian, WriteBytesExt};
use oot_explorer_core::gbi;

use crate::FLAGS_LIT;

pub trait LitVertex {
    fn position(&self) -> [i16; 3];
    fn texcoord(&self) -> [i16; 2];
    fn normal(&self) -> [i8; 3];
    fn alpha(&self) -> u8;
}
impl<'a> LitVertex for gbi::LitVertex<'a> {
    fn position(&self) -> [i16; 3] {
        gbi::LitVertex::position(*self)
    }
    fn texcoord(&self) -> [i16; 2] {
        gbi::LitVertex::texcoord(*self)
    }
    fn normal(&self) -> [i8; 3] {
        gbi::LitVertex::normal(*self)
    }
    fn alpha(&self) -> u8 {
        gbi::LitVertex::alpha(*self)
    }
}

pub fn write_lit_vertex<T>(dst: &mut [u8; 20], vertex: &T)
where
    T: LitVertex,
{
    let mut w = &mut dst[..];
    // [0..=5] Position
    let pos = vertex.position();
    w.write_i16::<NativeEndian>(pos[0]).unwrap();
    w.write_i16::<NativeEndian>(pos[1]).unwrap();
    w.write_i16::<NativeEndian>(pos[2]).unwrap();
    // [6..=7] Padding
    w.write_u16::<NativeEndian>(0).unwrap();
    // [8..=10] Normal
    let normal = vertex.normal();
    w.write_i8(normal[0]).unwrap();
    w.write_i8(normal[1]).unwrap();
    w.write_i8(normal[2]).unwrap();
    // [11] Flags
    w.write_u8(FLAGS_LIT).unwrap();
    // [12..=15] Texture coordinates
    let texcoord = vertex.texcoord();
    w.write_i16::<NativeEndian>(texcoord[0]).unwrap();
    w.write_i16::<NativeEndian>(texcoord[1]).unwrap();
    // [16..=19] Color (RGB are unused for lit geometry)
    w.write_u8(0).unwrap();
    w.write_u8(0).unwrap();
    w.write_u8(0).unwrap();
    w.write_u8(vertex.alpha()).unwrap();
    assert_eq!(w.len(), 0);
}
