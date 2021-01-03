use crate::fs::VromAddr;
use crate::header::{self, SceneHeader};
use crate::slice::StructReader;
use byteorder::{BigEndian, ReadBytesExt};
use std::ops::Range;

#[derive(Clone, Copy)]
pub struct Scene<'a> {
    addr: VromAddr,
    data: &'a [u8],
}
impl<'a> Scene<'a> {
    pub fn new(addr: VromAddr, data: &'a [u8]) -> Scene<'a> {
        Scene { addr, data }
    }
    pub fn addr(self) -> VromAddr {
        self.addr
    }
    pub fn vrom_range(self) -> Range<VromAddr> {
        self.addr..(self.addr + self.data.len() as u32)
    }
    pub fn data(self) -> &'a [u8] {
        self.data
    }
    pub fn headers(self) -> impl Iterator<Item = SceneHeader<'a>> {
        header::Iter::new(self.data).map(|header| header.scene_header())
    }
}

#[derive(Clone, Copy)]
pub struct Lighting<'a> {
    data: &'a [u8],
}
impl<'a> std::fmt::Debug for Lighting<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Lighting")
            .field("ambient_color", &self.ambient_color())
            .field("diffuse_color_a", &self.diffuse_color_a())
            .field("diffuse_color_b", &self.diffuse_color_b())
            .field("fog_color", &self.fog_color())
            .field("fog_start", &self.fog_start())
            .field("flags", &self.flags())
            .field("draw_distance", &self.draw_distance())
            .finish()
    }
}
impl<'a> StructReader<'a> for Lighting<'a> {
    const SIZE: usize = 0x16;

    fn new(data: &'a [u8]) -> Lighting<'a> {
        Lighting { data }
    }
}
impl<'a> Lighting<'a> {
    pub fn ambient_color(self) -> RgbColor {
        RgbColor {
            r: self.data[0x00],
            g: self.data[0x01],
            b: self.data[0x02],
        }
    }
    pub fn diffuse_color_a(self) -> RgbColor {
        RgbColor {
            r: self.data[0x03],
            g: self.data[0x04],
            b: self.data[0x05],
        }
    }
    pub fn diffuse_direction_a(self) -> [i8; 3] {
        [
            self.data[0x06] as i8,
            self.data[0x07] as i8,
            self.data[0x08] as i8,
        ]
    }
    pub fn diffuse_color_b(self) -> RgbColor {
        RgbColor {
            r: self.data[0x09],
            g: self.data[0x0a],
            b: self.data[0x0b],
        }
    }
    pub fn diffuse_direction_b(self) -> [i8; 3] {
        [
            self.data[0x0c] as i8,
            self.data[0x0d] as i8,
            self.data[0x0e] as i8,
        ]
    }
    pub fn fog_color(self) -> RgbColor {
        RgbColor {
            r: self.data[0x0f],
            g: self.data[0x10],
            b: self.data[0x11],
        }
    }
    pub fn fog_start(self) -> u16 {
        (&self.data[0x12..]).read_u16::<BigEndian>().unwrap() & 0x03ff
    }
    pub fn flags(self) -> u16 {
        (&self.data[0x12..]).read_u16::<BigEndian>().unwrap() >> 10
    }
    pub fn draw_distance(self) -> u16 {
        (&self.data[0x14..]).read_u16::<BigEndian>().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl<'a> std::fmt::Debug for RgbColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
