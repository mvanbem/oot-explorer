use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::fmt::{self, Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, Not};

use crate::segment::SegmentAddr;
use crate::slice::StructReader;

#[derive(Clone, Copy)]
pub struct DisplayList<'a> {
    data: &'a [u8],
}

impl<'a> DisplayList<'a> {
    pub fn new(data: &'a [u8]) -> DisplayList<'a> {
        DisplayList { data }
    }
    pub fn parse<F>(self, mut f: F)
    where
        F: FnMut(Instruction),
    {
        let mut data = self.data;
        loop {
            let instruction = Instruction::parse(&data[..Instruction::SIZE]);
            match instruction {
                Instruction::Dl { jump: true, .. } | Instruction::EndDl => break,
                _ => (),
            }
            f(instruction);
            data = &data[Instruction::SIZE..];
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    // 0x00
    Noop {
        tag: SegmentAddr,
    },
    // 0x01
    Vtx {
        count: u8,
        index: u8,
        ptr: SegmentAddr,
    },
    // 0x03
    CullDl {
        first: u8,
        last: u8,
    },
    // 0x04
    BranchZ {
        index: [u8; 2],
        compare: u32,
    },
    // 0x05
    Tri1 {
        index: [u8; 3],
    },
    // 0x06
    Tri2 {
        index_a: [u8; 3],
        index_b: [u8; 3],
    },
    // 0xd7
    Texture {
        max_lod: u8,
        tile: u8,
        enable: bool,
        scale_s: Qu0_16,
        scale_t: Qu0_16,
    },
    // 0xd9
    GeometryMode {
        clear_bits: GeometryMode,
        set_bits: GeometryMode,
    },
    // 0xda
    Mtx {
        len: u16,
        flags: MtxFlags,
        ptr: SegmentAddr,
    },
    // 0xde
    Dl {
        jump: bool,
        ptr: SegmentAddr,
    },
    // 0xdf
    EndDl,
    // 0xe1
    RdpHalf1 {
        word: u32,
    },
    // 0xe2
    SetOtherModeL {
        clear_bits: OtherModeLMask,
        set_bits: OtherModeL,
    },
    // 0xe3
    SetOtherModeH {
        clear_bits: OtherModeHMask,
        set_bits: OtherModeH,
    },
    // 0xe6
    RdpLoadSync,
    // 0xe7
    RdpPipeSync,
    // 0xe8
    RdpTileSync,
    // 0xf0
    LoadTlut {
        tile: u8,
        count: u16,
    },
    // 0xf2
    SetTileSize {
        start_s: Qu10_2,
        start_t: Qu10_2,
        tile: u8,
        end_s: Qu10_2,
        end_t: Qu10_2,
    },
    // 0xf3
    LoadBlock {
        start_s: Qu10_2,
        start_t: Qu10_2,
        tile: u8,
        texels: u16,
        dxt: Qu1_11,
    },
    // 0xf5
    SetTile {
        format: TextureFormat,
        depth: TextureDepth,
        stride: u16,
        addr: u16,
        tile: u8,
        palette: u8,
        clamp_t: bool,
        mirror_t: bool,
        mask_t: u8,
        shift_t: u8,
        clamp_s: bool,
        mirror_s: bool,
        mask_s: u8,
        shift_s: u8,
    },
    // 0xfa
    SetPrimColor {
        min_lod: u8,
        lod_fraction: u8,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    // 0xfb
    SetEnvColor {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    // 0xfc
    SetCombine {
        color_0: ColorCombine,
        alpha_0: AlphaCombine,
        color_1: ColorCombine,
        alpha_1: AlphaCombine,
    },
    // 0xfd
    SetTimg {
        format: TextureFormat,
        depth: TextureDepth,
        width: usize,
        ptr: SegmentAddr,
    },
}

impl Instruction {
    pub const SIZE: usize = 8;

    pub fn parse(data: &[u8]) -> Instruction {
        let u32_a = (&data[..]).read_u32::<BigEndian>().unwrap() & 0x00ff_ffff;
        let u32_b = (&data[4..]).read_u32::<BigEndian>().unwrap();
        match data[0] {
            0x00 => Instruction::Noop {
                tag: SegmentAddr(u32_b),
            },
            0x01 => {
                let count = (u32_a >> 12) as u8;
                Instruction::Vtx {
                    count,
                    index: ((u32_a >> 1) as u8).wrapping_sub(count),
                    ptr: SegmentAddr(u32_b),
                }
            }
            0x03 => Instruction::CullDl {
                first: (u32_a >> 1) as u8,
                last: (u32_b >> 1) as u8,
            },
            0x04 => Instruction::BranchZ {
                index: [((u32_a >> 12) & 0xff) as u8, (u32_a & 0xff) as u8],
                compare: u32_b,
            },
            0x05 => Instruction::Tri1 {
                index: [data[1] / 2, data[2] / 2, data[3] / 2],
            },
            0x06 => Instruction::Tri2 {
                index_a: [data[1] / 2, data[2] / 2, data[3] / 2],
                index_b: [data[5] / 2, data[6] / 2, data[7] / 2],
            },
            0xd7 => Instruction::Texture {
                max_lod: ((u32_a >> 11) & 0x07) as u8,
                tile: ((u32_a >> 8) & 0x07) as u8,
                enable: ((u32_a >> 1) & 0x7f) == 1,
                scale_s: Qu0_16((u32_b >> 16) as u16),
                scale_t: Qu0_16(u32_b as u16),
            },
            0xd9 => Instruction::GeometryMode {
                clear_bits: GeometryMode(!u32_a & 0x00ff_ffff),
                set_bits: GeometryMode(u32_b),
            },
            0xda => Instruction::Mtx {
                len: ((((u32_a >> 19) & 0x1f) << 3) + 1) as u16,
                flags: MtxFlags(data[3]),
                ptr: SegmentAddr(u32_b),
            },
            0xde => Instruction::Dl {
                jump: data[1] == 0x01,
                ptr: SegmentAddr(u32_b),
            },
            0xdf => Instruction::EndDl,
            0xe1 => Instruction::RdpHalf1 { word: u32_b },
            0xe2 => {
                let width = data[3] + 1;
                let shift = 32 - width - data[2];
                Instruction::SetOtherModeL {
                    clear_bits: OtherModeLMask(((1 << width) - 1) << shift),
                    set_bits: OtherModeL(u32_b),
                }
            }
            0xe3 => {
                let width = data[3] + 1;
                let shift = 32 - width - data[2];
                Instruction::SetOtherModeH {
                    clear_bits: OtherModeHMask(((1 << width) - 1) << shift),
                    set_bits: OtherModeH(u32_b),
                }
            }
            0xe6 => Instruction::RdpLoadSync,
            0xe7 => Instruction::RdpPipeSync,
            0xe8 => Instruction::RdpTileSync,
            0xf0 => Instruction::LoadTlut {
                tile: ((u32_b >> 24) & 0x07) as u8,
                count: ((u32_b >> 14) & 0x03ff) as u16 + 1,
            },
            0xf2 => Instruction::SetTileSize {
                start_s: Qu10_2(((u32_a >> 12) & 0x0fff) as u16),
                start_t: Qu10_2((u32_a & 0x0fff) as u16),
                tile: ((u32_b >> 24) & 0x07) as u8,
                end_s: Qu10_2(((u32_b >> 12) & 0x0fff) as u16),
                end_t: Qu10_2((u32_b & 0x0fff) as u16),
            },
            0xf3 => Instruction::LoadBlock {
                start_s: Qu10_2(((u32_a >> 12) & 0x0fff) as u16),
                start_t: Qu10_2((u32_a & 0x0fff) as u16),
                tile: ((u32_b >> 24) & 0x07) as u8,
                texels: ((u32_b >> 12) & 0x0fff) as u16 + 1,
                dxt: Qu1_11((u32_b & 0x0fff) as u16),
            },
            0xf5 => Instruction::SetTile {
                format: TextureFormat::parse(data[1] >> 5),
                depth: TextureDepth::parse((data[1] >> 3) & 0x03),
                stride: ((u32_a >> 9) & 0x01ff) as u16,
                addr: (u32_a & 0x01ff) as u16,
                tile: ((u32_b >> 24) & 0x07) as u8,
                palette: ((u32_b >> 20) & 0x0f) as u8,
                clamp_t: ((u32_b >> 19) & 1) == 1,
                mirror_t: ((u32_b >> 18) & 1) == 1,
                mask_t: ((u32_b >> 14) & 0x0f) as u8,
                shift_t: ((u32_b >> 10) & 0x0f) as u8,
                clamp_s: ((u32_b >> 9) & 1) == 1,
                mirror_s: ((u32_b >> 8) & 1) == 1,
                mask_s: ((u32_b >> 4) & 0x0f) as u8,
                shift_s: (u32_b & 0x0f) as u8,
            },
            0xfa => Instruction::SetPrimColor {
                min_lod: data[2],
                lod_fraction: data[3],
                r: data[4],
                g: data[5],
                b: data[6],
                a: data[7],
            },
            0xfb => Instruction::SetEnvColor {
                r: data[4],
                g: data[5],
                b: data[6],
                a: data[7],
            },
            0xfc => Instruction::SetCombine {
                color_0: ColorCombine {
                    a: ColorInput::parse_a(((u32_a >> 20) & 0x0f) as u8),
                    b: ColorInput::parse_b(((u32_b >> 28) & 0x0f) as u8),
                    c: ColorInput::parse_c(((u32_a >> 15) & 0x1f) as u8),
                    d: ColorInput::parse_d(((u32_b >> 15) & 0x07) as u8),
                },
                alpha_0: AlphaCombine {
                    a: AlphaInput::parse_abd(((u32_a >> 12) & 0x07) as u8),
                    b: AlphaInput::parse_abd(((u32_b >> 12) & 0x07) as u8),
                    c: AlphaInput::parse_c(((u32_a >> 9) & 0x07) as u8),
                    d: AlphaInput::parse_abd(((u32_b >> 9) & 0x07) as u8),
                },
                color_1: ColorCombine {
                    a: ColorInput::parse_a(((u32_a >> 5) & 0x0f) as u8),
                    b: ColorInput::parse_b(((u32_b >> 24) & 0x0f) as u8),
                    c: ColorInput::parse_c((u32_a & 0x1f) as u8),
                    d: ColorInput::parse_d(((u32_b >> 6) & 0x07) as u8),
                },
                alpha_1: AlphaCombine {
                    a: AlphaInput::parse_abd(((u32_b >> 21) & 0x07) as u8),
                    b: AlphaInput::parse_abd(((u32_b >> 3) & 0x07) as u8),
                    c: AlphaInput::parse_c(((u32_b >> 18) & 0x07) as u8),
                    d: AlphaInput::parse_abd((u32_b & 0x07) as u8),
                },
            },
            0xfd => Instruction::SetTimg {
                format: TextureFormat::parse(data[1] >> 5),
                depth: TextureDepth::parse((data[1] >> 3) & 0x03),
                width: (u32_a & 0x0fff) as usize + 1,
                ptr: SegmentAddr(u32_b),
            },
            opcode => panic!("unexpected GBI opcode: 0x{:02x}", opcode),
        }
    }
}

/// A fixed-point 0.16 number.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Qu0_16(pub u16);

impl Debug for Qu0_16 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Qu0_16({})", self.as_f32())
    }
}

impl Qu0_16 {
    pub fn as_f32(self) -> f32 {
        self.0 as f32 / 65536.0
    }

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / 65536.0
    }
}

/// A fixed-point 10.2 number.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Qu10_2(pub u16);

impl Debug for Qu10_2 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Qu10_2({})", self.as_f32())
    }
}

impl Qu10_2 {
    pub fn as_f32(self) -> f32 {
        (self.0 & 0x0fff) as f32 / 4.0
    }

    pub fn as_f64(self) -> f64 {
        (self.0 & 0x0fff) as f64 / 4.0
    }
}

/// A fixed-point 10.2 number.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Qu1_11(pub u16);

impl Debug for Qu1_11 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Qu1_11({})", self.as_f32())
    }
}

impl Qu1_11 {
    pub fn as_f32(self) -> f32 {
        (self.0 & 0x0fff) as f32 / 2048.0
    }

    pub fn as_f64(self) -> f64 {
        (self.0 & 0x0fff) as f64 / 2048.0
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
)]
pub struct MtxFlags(pub u8);

impl Debug for MtxFlags {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            if self.test(MtxFlags::PROJECTION) {
                "PROJECTION"
            } else {
                "MODELVIEW"
            },
            if self.test(MtxFlags::LOAD) {
                "LOAD"
            } else {
                "MUL"
            },
        )?;
        if self.test(MtxFlags::PUSH) {
            write!(f, " | PUSH")?;
        }
        Ok(())
    }
}

impl MtxFlags {
    pub const NOPUSH: MtxFlags = MtxFlags(0x00);
    pub const PUSH: MtxFlags = MtxFlags(0x01);

    pub const MUL: MtxFlags = MtxFlags(0x00);
    pub const LOAD: MtxFlags = MtxFlags(0x02);

    pub const MODELVIEW: MtxFlags = MtxFlags(0x00);
    pub const PROJECTION: MtxFlags = MtxFlags(0x04);

    pub const ALL: MtxFlags = MtxFlags(0x07);

    pub fn test(self, mask: MtxFlags) -> bool {
        (self & mask) == mask
    }
}

impl Not for MtxFlags {
    type Output = MtxFlags;
    fn not(self) -> MtxFlags {
        self ^ MtxFlags::ALL
    }
}

#[derive(Clone, Debug)]
struct Separator {
    sep: &'static str,
    any: bool,
}

impl Separator {
    fn new(sep: &'static str) -> Separator {
        Separator { sep, any: false }
    }

    fn write(&mut self, f: &mut Formatter) -> fmt::Result {
        if self.any {
            write!(f, "{}", self.sep)
        } else {
            self.any = true;
            Ok(())
        }
    }

    fn none(&self) -> bool {
        !self.any
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
    derive_more::Not,
)]
pub struct GeometryMode(pub u32);

impl Debug for GeometryMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut sep = Separator::new(" | ");
        for name in [
            (GeometryMode::ZBUFFER, "ZBUFFER"),
            (GeometryMode::SHADE, "SHADE"),
            (GeometryMode::CULL_FRONT, "CULL_FRONT"),
            (GeometryMode::CULL_BACK, "CULL_BACK"),
            (GeometryMode::FOG, "FOG"),
            (GeometryMode::LIGHTING, "LIGHTING"),
            (GeometryMode::TEXTURE_GEN, "TEXTURE_GEN"),
            (GeometryMode::TEXTURE_GEN_LINEAR, "TEXTURE_GEN_LINEAR"),
            (GeometryMode::SHADING_SMOOTH, "SHADING_SMOOTH"),
            (GeometryMode::CLIPPING, "CLIPPING"),
        ]
        .iter()
        .flat_map(
            |(mask, name)| {
                if self.test(*mask) {
                    Some(name)
                } else {
                    None
                }
            },
        ) {
            sep.write(f)?;
            write!(f, "{}", name)?;
        }
        let unknown = self.unknown();
        if unknown != 0 {
            sep.write(f)?;
            write!(f, "0x{:08x}", unknown)?;
        }
        if sep.none() {
            write!(f, "0")?;
        }
        Ok(())
    }
}

impl Default for GeometryMode {
    fn default() -> GeometryMode {
        GeometryMode::CLIPPING
    }
}

impl GeometryMode {
    pub const ZBUFFER: GeometryMode = GeometryMode(0x0000_0001);
    pub const SHADE: GeometryMode = GeometryMode(0x0000_0004);
    pub const CULL_FRONT: GeometryMode = GeometryMode(0x0000_0200);
    pub const CULL_BACK: GeometryMode = GeometryMode(0x0000_0400);
    pub const FOG: GeometryMode = GeometryMode(0x0001_0000);
    pub const LIGHTING: GeometryMode = GeometryMode(0x0002_0000);
    pub const TEXTURE_GEN: GeometryMode = GeometryMode(0x0004_0000);
    pub const TEXTURE_GEN_LINEAR: GeometryMode = GeometryMode(0x0008_0000);
    pub const SHADING_SMOOTH: GeometryMode = GeometryMode(0x0020_0000);
    pub const CLIPPING: GeometryMode = GeometryMode(0x0080_0000);

    pub fn test(self, mask: GeometryMode) -> bool {
        (self.0 & mask.0) == mask.0
    }

    pub fn unknown(self) -> u32 {
        (self
            & !(GeometryMode::ZBUFFER
                | GeometryMode::SHADE
                | GeometryMode::CULL_FRONT
                | GeometryMode::CULL_BACK
                | GeometryMode::FOG
                | GeometryMode::LIGHTING
                | GeometryMode::TEXTURE_GEN
                | GeometryMode::TEXTURE_GEN_LINEAR
                | GeometryMode::SHADING_SMOOTH
                | GeometryMode::CLIPPING))
            .0
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
    derive_more::Not,
)]
pub struct OtherModeL(pub u32);

impl Debug for OtherModeL {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut sep = Separator::new(" | ");
        for name in [
            match *self & OtherModeLMask::ALPHACOMPARE {
                x if x == OtherModeL::AC_NONE => Some("AC_NONE"),
                x if x == OtherModeL::AC_THRESHOLD => Some("AC_THRESHOLD"),
                x if x == OtherModeL::AC_DITHER => Some("AC_DITHER"),
                _ => Some("AC_UNKNOWN"),
            },
            match *self & OtherModeLMask::ZSRCSEL {
                x if x == OtherModeL::ZS_PIXEL => Some("ZS_PIXEL"),
                x if x == OtherModeL::ZS_PRIM => Some("ZS_PRIM"),
                _ => unreachable!(),
            },
            if self.test(OtherModeL::AA_EN) {
                Some("AA_EN")
            } else {
                None
            },
            if self.test(OtherModeL::Z_CMP) {
                Some("Z_CMP")
            } else {
                None
            },
            if self.test(OtherModeL::Z_UPD) {
                Some("Z_UPD")
            } else {
                None
            },
            if self.test(OtherModeL::IM_RD) {
                Some("IM_RD")
            } else {
                None
            },
            if self.test(OtherModeL::CLR_ON_CVG) {
                Some("CLR_ON_CVG")
            } else {
                None
            },
            match *self & OtherModeLMask::CVG_DST {
                x if x == OtherModeL::CVG_DST_CLAMP => Some("CVG_DST_CLAMP"),
                x if x == OtherModeL::CVG_DST_WRAP => Some("CVG_DST_WRAP"),
                x if x == OtherModeL::CVG_DST_FULL => Some("CVG_DST_FULL"),
                x if x == OtherModeL::CVG_DST_SAVE => Some("CVG_DST_SAVE"),
                _ => unreachable!(),
            },
            match *self & OtherModeLMask::ZMODE {
                x if x == OtherModeL::ZMODE_OPA => Some("ZMODE_OPA"),
                x if x == OtherModeL::ZMODE_INTER => Some("ZMODE_INTER"),
                x if x == OtherModeL::ZMODE_XLU => Some("ZMODE_XLU"),
                x if x == OtherModeL::ZMODE_DEC => Some("ZMODE_DEC"),
                _ => unreachable!(),
            },
            if self.test(OtherModeL::CVG_X_ALPHA) {
                Some("CVG_X_ALPHA")
            } else {
                None
            },
            if self.test(OtherModeL::ALPHA_CVG_SEL) {
                Some("ALPHA_CVG_SEL")
            } else {
                None
            },
            if self.test(OtherModeL::FORCE_BL) {
                Some("FORCE_BL")
            } else {
                None
            },
        ]
        .iter()
        .flatten()
        {
            sep.write(f)?;
            write!(f, "{}", name)?;
        }
        let unknown = self.unknown();
        if unknown != 0 {
            write!(f, " | 0x{:08x}", unknown)?;
        }
        Ok(())
    }
}

impl OtherModeL {
    pub const AC_NONE: OtherModeL = OtherModeL(0x0000_0000);
    pub const AC_THRESHOLD: OtherModeL = OtherModeL(0x0000_0001);
    pub const AC_DITHER: OtherModeL = OtherModeL(0x0000_0003);

    pub const ZS_PIXEL: OtherModeL = OtherModeL(0x0000_0000);
    pub const ZS_PRIM: OtherModeL = OtherModeL(0x0000_0004);

    pub const AA_EN: OtherModeL = OtherModeL(0x0000_0008);
    pub const Z_CMP: OtherModeL = OtherModeL(0x0000_0010);
    pub const Z_UPD: OtherModeL = OtherModeL(0x0000_0020);
    pub const IM_RD: OtherModeL = OtherModeL(0x0000_0040);
    pub const CLR_ON_CVG: OtherModeL = OtherModeL(0x0000_0080);

    pub const CVG_DST_CLAMP: OtherModeL = OtherModeL(0x0000_0000);
    pub const CVG_DST_WRAP: OtherModeL = OtherModeL(0x0000_0100);
    pub const CVG_DST_FULL: OtherModeL = OtherModeL(0x0000_0200);
    pub const CVG_DST_SAVE: OtherModeL = OtherModeL(0x0000_0300);

    pub const ZMODE_OPA: OtherModeL = OtherModeL(0x0000_0000);
    pub const ZMODE_INTER: OtherModeL = OtherModeL(0x0000_0400);
    pub const ZMODE_XLU: OtherModeL = OtherModeL(0x0000_0800);
    pub const ZMODE_DEC: OtherModeL = OtherModeL(0x0000_0c00);

    pub const CVG_X_ALPHA: OtherModeL = OtherModeL(0x0000_1000);
    pub const ALPHA_CVG_SEL: OtherModeL = OtherModeL(0x0000_2000);
    pub const FORCE_BL: OtherModeL = OtherModeL(0x0000_4000);
    pub const OBSOLETE: OtherModeL = OtherModeL(0x0000_8000);

    pub fn test(self, mask: OtherModeL) -> bool {
        (self.0 & mask.0) == mask.0
    }

    pub fn unknown(self) -> u32 {
        (self
            & !(OtherModeLMask::ALPHACOMPARE
                | OtherModeLMask::ZSRCSEL
                | OtherModeLMask::RENDERMODE))
            .0
    }
}

impl std::ops::BitAnd<OtherModeLMask> for OtherModeL {
    type Output = OtherModeL;
    fn bitand(self, rhs: OtherModeLMask) -> OtherModeL {
        OtherModeL(self.0 & rhs.0)
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
    derive_more::Not,
)]
pub struct OtherModeLMask(pub u32);

impl Debug for OtherModeLMask {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut sep = Separator::new(" | ");
        if self.test(OtherModeLMask::ALPHACOMPARE) {
            sep.write(f)?;
            write!(f, "ALPHACOMPARE")?;
        }
        if self.test(OtherModeLMask::ZSRCSEL) {
            sep.write(f)?;
            write!(f, "ZSRCSEL")?;
        }
        if self.test(OtherModeLMask::RENDERMODE) {
            sep.write(f)?;
            write!(f, "RENDERMODE")?;
        } else {
            for name in [
                (OtherModeLMask::AA_EN, "AA_EN"),
                (OtherModeLMask::Z_CMP, "Z_CMP"),
                (OtherModeLMask::Z_UPD, "Z_UPD"),
                (OtherModeLMask::IM_RD, "IM_RD"),
                (OtherModeLMask::CLR_ON_CVG, "CLR_ON_CVG"),
                (OtherModeLMask::CVG_DST, "CVG_DST"),
                (OtherModeLMask::ZMODE, "ZMODE"),
                (OtherModeLMask::CVG_X_ALPHA, "CVG_X_ALPHA"),
                (OtherModeLMask::ALPHA_CVG_SEL, "ALPHA_CVG_SEL"),
                (OtherModeLMask::FORCE_BL, "FORCE_BL"),
            ]
            .iter()
            .flat_map(|(mask, name)| if self.test(*mask) { Some(*name) } else { None })
            {
                sep.write(f)?;
                write!(f, "{}", name)?;
            }
        }
        let unknown = self.unknown();
        if unknown != 0 {
            write!(f, " | 0x{:08x}", unknown)?;
        }
        Ok(())
    }
}

impl OtherModeLMask {
    pub const ALPHACOMPARE: OtherModeLMask = OtherModeLMask(0x0000_0003);
    pub const ZSRCSEL: OtherModeLMask = OtherModeLMask(0x0000_0003);
    pub const RENDERMODE: OtherModeLMask = OtherModeLMask(0xffff_fff8);

    // Subsets of RENDERMODE
    pub const AA_EN: OtherModeLMask = OtherModeLMask(0x0000_0008);
    pub const Z_CMP: OtherModeLMask = OtherModeLMask(0x0000_0010);
    pub const Z_UPD: OtherModeLMask = OtherModeLMask(0x0000_0020);
    pub const IM_RD: OtherModeLMask = OtherModeLMask(0x0000_0040);
    pub const CLR_ON_CVG: OtherModeLMask = OtherModeLMask(0x0000_0080);
    pub const CVG_DST: OtherModeLMask = OtherModeLMask(0x0000_0300);
    pub const ZMODE: OtherModeLMask = OtherModeLMask(0x0000_0c00);
    pub const CVG_X_ALPHA: OtherModeLMask = OtherModeLMask(0x0000_1000);
    pub const ALPHA_CVG_SEL: OtherModeLMask = OtherModeLMask(0x0000_2000);
    pub const FORCE_BL: OtherModeLMask = OtherModeLMask(0x0000_4000);
    pub const OBSOLETE: OtherModeLMask = OtherModeLMask(0x0000_8000);

    pub fn test(self, mask: OtherModeLMask) -> bool {
        (self.0 & mask.0) == mask.0
    }

    pub fn unknown(self) -> u32 {
        (self
            & !(OtherModeLMask::ALPHACOMPARE
                | OtherModeLMask::ZSRCSEL
                | OtherModeLMask::RENDERMODE))
            .0
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
    derive_more::Not,
)]
pub struct OtherModeH(pub u32);

impl Debug for OtherModeH {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut sep = Separator::new(" | ");
        for name in [
            match *self & OtherModeHMask::ALPHADITHER {
                x if x == OtherModeH::AD_PATTERN => Some("AD_PATTERN"),
                x if x == OtherModeH::AD_NOTPATTERN => Some("AD_NOTPATTERN"),
                x if x == OtherModeH::AD_NOISE => Some("AD_NOISE"),
                x if x == OtherModeH::AD_DISABLE => None,
                _ => unreachable!(),
            },
            match *self & OtherModeHMask::RGBDITHER {
                x if x == OtherModeH::CD_MAGICSQ => Some("CD_MAGICSQ"),
                x if x == OtherModeH::CD_BAYER => Some("CD_BAYER"),
                x if x == OtherModeH::CD_NOISE => Some("CD_NOISE"),
                x if x == OtherModeH::CD_DISABLE => None,
                _ => unreachable!(),
            },
            if self.test(OtherModeH::COMBKEY) {
                Some("COMBKEY")
            } else {
                None
            },
            match *self & OtherModeHMask::TEXTCONV {
                x if x == OtherModeH::TC_CONV => Some("TC_CONV"),
                x if x == OtherModeH::TC_FILTCONV => Some("TC_FILTCONV"),
                x if x == OtherModeH::TC_FILT => Some("TC_FILT"),
                _ => Some("TC_UNKNOWN"),
            },
            match *self & OtherModeHMask::TEXTFILT {
                x if x == OtherModeH::TF_POINT => Some("TF_POINT"),
                x if x == OtherModeH::TF_BILERP => Some("TF_BILERP"),
                x if x == OtherModeH::TF_AVERAGE => Some("TF_AVERAGE"),
                _ => Some("TF_UNKNOWN"),
            },
            match *self & OtherModeHMask::TEXTLUT {
                x if x == OtherModeH::TT_NONE => None,
                x if x == OtherModeH::TT_RGBA16 => Some("TT_RGBA16"),
                x if x == OtherModeH::TT_IA16 => Some("TT_IA16"),
                _ => Some("TT_UNKNOWN"),
            },
            match *self & OtherModeHMask::TEXTLOD {
                x if x == OtherModeH::TL_TILE => Some("TL_TILE"),
                x if x == OtherModeH::TL_LOD => Some("TL_LOD"),
                _ => unreachable!(),
            },
            match *self & OtherModeHMask::TEXTDETAIL {
                x if x == OtherModeH::TD_CLAMP => Some("TD_CLAMP"),
                x if x == OtherModeH::TD_SHARPEN => Some("TD_SHARPEN"),
                x if x == OtherModeH::TD_DETAIL => Some("TD_DETAIL"),
                _ => Some("TD_UNKNOWN"),
            },
            if self.test(OtherModeH::TEXTPERSP) {
                Some("TEXTPERSP")
            } else {
                None
            },
            match *self & OtherModeHMask::CYCLETYPE {
                x if x == OtherModeH::CYC_1CYCLE => Some("CYC_1CYCLE"),
                x if x == OtherModeH::CYC_2CYCLE => Some("CYC_2CYCLE"),
                x if x == OtherModeH::CYC_COPY => Some("CYC_COPY"),
                x if x == OtherModeH::CYC_FILL => Some("CYC_FILL"),
                _ => unreachable!(),
            },
            if self.test(OtherModeH::V1_COLORDITHER) {
                Some("V1_COLORDITHER")
            } else {
                None
            },
            match *self & OtherModeHMask::PIPELINE {
                x if x == OtherModeH::PM_NPRIMITIVE => Some("PM_NPRIMITIVE"),
                x if x == OtherModeH::PM_1PRIMITIVE => Some("PM_1PRIMITIVE"),
                _ => unreachable!(),
            },
        ]
        .iter()
        .flatten()
        {
            sep.write(f)?;
            write!(f, "{}", name)?;
        }
        let unknown = self.unknown();
        if unknown != 0 {
            write!(f, " | 0x{:08x}", unknown)?;
        }
        Ok(())
    }
}

impl OtherModeH {
    pub const AD_PATTERN: OtherModeH = OtherModeH(0x0000_0000);
    pub const AD_NOTPATTERN: OtherModeH = OtherModeH(0x0000_0010);
    pub const AD_NOISE: OtherModeH = OtherModeH(0x0000_0020);
    pub const AD_DISABLE: OtherModeH = OtherModeH(0x0000_0030);

    pub const CD_MAGICSQ: OtherModeH = OtherModeH(0x0000_0000);
    pub const CD_BAYER: OtherModeH = OtherModeH(0x0000_0040);
    pub const CD_NOISE: OtherModeH = OtherModeH(0x0000_0080);
    pub const CD_DISABLE: OtherModeH = OtherModeH(0x0000_00c0);

    pub const COMBKEY: OtherModeH = OtherModeH(0x0000_0100);

    pub const TC_CONV: OtherModeH = OtherModeH(0x0000_0000);
    pub const TC_FILTCONV: OtherModeH = OtherModeH(0x0000_0a00);
    pub const TC_FILT: OtherModeH = OtherModeH(0x0000_0c00);

    pub const TF_POINT: OtherModeH = OtherModeH(0x0000_0000);
    pub const TF_BILERP: OtherModeH = OtherModeH(0x0000_2000);
    pub const TF_AVERAGE: OtherModeH = OtherModeH(0x0000_3000);

    pub const TT_NONE: OtherModeH = OtherModeH(0x0000_0000);
    pub const TT_RGBA16: OtherModeH = OtherModeH(0x0000_8000);
    pub const TT_IA16: OtherModeH = OtherModeH(0x0000_c000);

    pub const TL_TILE: OtherModeH = OtherModeH(0x0000_0000);
    pub const TL_LOD: OtherModeH = OtherModeH(0x0001_0000);

    pub const TD_CLAMP: OtherModeH = OtherModeH(0x0000_0000);
    pub const TD_SHARPEN: OtherModeH = OtherModeH(0x0002_0000);
    pub const TD_DETAIL: OtherModeH = OtherModeH(0x0004_0000);

    pub const TEXTPERSP: OtherModeH = OtherModeH(0x0008_0000);

    pub const CYC_1CYCLE: OtherModeH = OtherModeH(0x0000_0000);
    pub const CYC_2CYCLE: OtherModeH = OtherModeH(0x0010_0000);
    pub const CYC_COPY: OtherModeH = OtherModeH(0x0020_0000);
    pub const CYC_FILL: OtherModeH = OtherModeH(0x0030_0000);

    pub const V1_COLORDITHER: OtherModeH = OtherModeH(0x0040_0000);

    pub const PM_NPRIMITIVE: OtherModeH = OtherModeH(0x0000_0000);
    pub const PM_1PRIMITIVE: OtherModeH = OtherModeH(0x0080_0000);

    pub fn test(self, mask: OtherModeH) -> bool {
        (self.0 & mask.0) == mask.0
    }

    pub fn unknown(self) -> u32 {
        (self
            & !(OtherModeHMask::ALPHADITHER
                | OtherModeHMask::RGBDITHER
                | OtherModeHMask::COMBKEY
                | OtherModeHMask::TEXTCONV
                | OtherModeHMask::TEXTFILT
                | OtherModeHMask::TEXTLUT
                | OtherModeHMask::TEXTLOD
                | OtherModeHMask::TEXTDETAIL
                | OtherModeHMask::TEXTPERSP
                | OtherModeHMask::CYCLETYPE
                | OtherModeHMask::V1_COLORDITHER
                | OtherModeHMask::PIPELINE))
            .0
    }
}

impl BitAnd<OtherModeHMask> for OtherModeH {
    type Output = OtherModeH;
    fn bitand(self, rhs: OtherModeHMask) -> OtherModeH {
        OtherModeH(self.0 & rhs.0)
    }
}

impl BitAndAssign<OtherModeHMask> for OtherModeH {
    fn bitand_assign(&mut self, rhs: OtherModeHMask) {
        self.0 &= rhs.0;
    }
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
    derive_more::Not,
)]
pub struct OtherModeHMask(pub u32);

impl Debug for OtherModeHMask {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut sep = Separator::new(" | ");
        for name in [
            (OtherModeHMask::ALPHADITHER, "ALPHADITHER"),
            (OtherModeHMask::RGBDITHER, "RGBDITHER"),
            (OtherModeHMask::COMBKEY, "COMBKEY"),
            (OtherModeHMask::TEXTCONV, "TEXTCONV"),
            (OtherModeHMask::TEXTFILT, "TEXTFILT"),
            (OtherModeHMask::TEXTLUT, "TEXTLUT"),
            (OtherModeHMask::TEXTLOD, "TEXTLOD"),
            (OtherModeHMask::TEXTDETAIL, "TEXTDETAIL"),
            (OtherModeHMask::TEXTPERSP, "TEXTPERSP"),
            (OtherModeHMask::CYCLETYPE, "CYCLETYPE"),
            (OtherModeHMask::V1_COLORDITHER, "V1_COLORDITHER"),
            (OtherModeHMask::PIPELINE, "PIPELINE"),
        ]
        .iter()
        .flat_map(
            |(mask, name)| {
                if self.test(*mask) {
                    Some(*name)
                } else {
                    None
                }
            },
        ) {
            sep.write(f)?;
            write!(f, "{}", name)?;
        }
        let unknown = self.unknown();
        if unknown != 0 {
            write!(f, " | 0x{:08x}", unknown)?;
        }
        Ok(())
    }
}

impl OtherModeHMask {
    pub const ALPHADITHER: OtherModeHMask = OtherModeHMask(0x0000_0030);
    pub const RGBDITHER: OtherModeHMask = OtherModeHMask(0x0000_00c0);
    pub const COMBKEY: OtherModeHMask = OtherModeHMask(0x0000_0100);
    pub const TEXTCONV: OtherModeHMask = OtherModeHMask(0x0000_0e00);
    pub const TEXTFILT: OtherModeHMask = OtherModeHMask(0x0000_3000);
    pub const TEXTLUT: OtherModeHMask = OtherModeHMask(0x0000_c000);
    pub const TEXTLOD: OtherModeHMask = OtherModeHMask(0x0001_0000);
    pub const TEXTDETAIL: OtherModeHMask = OtherModeHMask(0x0006_0000);
    pub const TEXTPERSP: OtherModeHMask = OtherModeHMask(0x0008_0000);
    pub const CYCLETYPE: OtherModeHMask = OtherModeHMask(0x0030_0000);
    pub const V1_COLORDITHER: OtherModeHMask = OtherModeHMask(0x0040_0000);
    pub const PIPELINE: OtherModeHMask = OtherModeHMask(0x0080_0000);

    pub fn test(self, mask: OtherModeHMask) -> bool {
        (self.0 & mask.0) == mask.0
    }

    pub fn unknown(self) -> u32 {
        (self
            & !(OtherModeHMask::ALPHADITHER
                | OtherModeHMask::RGBDITHER
                | OtherModeHMask::COMBKEY
                | OtherModeHMask::TEXTCONV
                | OtherModeHMask::TEXTFILT
                | OtherModeHMask::TEXTLUT
                | OtherModeHMask::TEXTLOD
                | OtherModeHMask::TEXTDETAIL
                | OtherModeHMask::TEXTPERSP
                | OtherModeHMask::CYCLETYPE
                | OtherModeHMask::V1_COLORDITHER
                | OtherModeHMask::PIPELINE))
            .0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextureFormat {
    Rgba,
    Yuv,
    Ci,
    Ia,
    I,
}

impl TextureFormat {
    fn parse(value: u8) -> TextureFormat {
        match value {
            0 => TextureFormat::Rgba,
            1 => TextureFormat::Yuv,
            2 => TextureFormat::Ci,
            3 => TextureFormat::Ia,
            4 => TextureFormat::I,
            _ => panic!("unexpected texture format: 0x{:02x}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextureDepth {
    Bits4,
    Bits8,
    Bits16,
    Bits32,
}

impl TextureDepth {
    fn parse(value: u8) -> TextureDepth {
        match value {
            0 => TextureDepth::Bits4,
            1 => TextureDepth::Bits8,
            2 => TextureDepth::Bits16,
            3 => TextureDepth::Bits32,
            _ => panic!("unexpected texture depth: 0x{:02x}", value),
        }
    }

    pub fn texels_per_tmem_word<T: FromPrimitive>(self) -> T {
        match self {
            TextureDepth::Bits4 => T::from_u8(16).unwrap(),
            TextureDepth::Bits8 => T::from_u8(8).unwrap(),
            TextureDepth::Bits16 => T::from_u8(4).unwrap(),
            TextureDepth::Bits32 => T::from_u8(2).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, derive_more::BitAnd, derive_more::BitOr)]
pub struct CombinerReference(pub u32);

impl CombinerReference {
    pub const COMBINED: CombinerReference = CombinerReference(0x0000_0001);
    pub const TEXEL_0: CombinerReference = CombinerReference(0x0000_0002);
    pub const TEXEL_1: CombinerReference = CombinerReference(0x0000_0004);
    pub const PRIMITIVE: CombinerReference = CombinerReference(0x0000_0008);
    pub const SHADE: CombinerReference = CombinerReference(0x0000_0010);
    pub const ENVIRONMENT: CombinerReference = CombinerReference(0x0000_0020);
    pub const NOISE: CombinerReference = CombinerReference(0x0000_0040);
    pub const CENTER: CombinerReference = CombinerReference(0x0000_0080);
    pub const K_4: CombinerReference = CombinerReference(0x0000_0100);
    pub const SCALE: CombinerReference = CombinerReference(0x0000_0200);
    pub const LOD_FRACTION: CombinerReference = CombinerReference(0x0000_0400);
    pub const PRIM_LOD_FRAC: CombinerReference = CombinerReference(0x0000_0800);
    pub const K_5: CombinerReference = CombinerReference(0x0000_1000);

    pub fn test(self, mask: CombinerReference) -> bool {
        (self & mask) == mask
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ColorCombine {
    pub a: ColorInput,
    pub b: ColorInput,
    pub c: ColorInput,
    pub d: ColorInput,
}

impl Debug for ColorCombine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[({:?} - {:?}) * {:?} + {:?}]",
            self.a, self.b, self.c, self.d,
        )
    }
}

impl ColorCombine {
    pub fn new(a: ColorInput, b: ColorInput, c: ColorInput, d: ColorInput) -> ColorCombine {
        ColorCombine { a, b, c, d }
    }

    pub fn references(self) -> CombinerReference {
        self.a.references() | self.b.references() | self.c.references() | self.d.references()
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct AlphaCombine {
    pub a: AlphaInput,
    pub b: AlphaInput,
    pub c: AlphaInput,
    pub d: AlphaInput,
}

impl Debug for AlphaCombine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[({:?} - {:?}) * {:?} + {:?}]",
            self.a, self.b, self.c, self.d,
        )
    }
}

impl AlphaCombine {
    pub fn references(self) -> CombinerReference {
        self.a.references() | self.b.references() | self.c.references() | self.d.references()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorInput {
    Combined,
    Texel0,
    Texel1,
    Primitive,
    Shade,
    Environment,
    One,
    Noise,
    Zero,
    Center,
    K4,
    Scale,
    CombinedAlpha,
    Texel0Alpha,
    Texel1Alpha,
    PrimitiveAlpha,
    ShadeAlpha,
    EnvAlpha,
    LodFraction,
    PrimLodFrac,
    K5,
}

impl ColorInput {
    fn parse_a(value: u8) -> ColorInput {
        match value {
            0x00 => ColorInput::Combined,
            0x01 => ColorInput::Texel0,
            0x02 => ColorInput::Texel1,
            0x03 => ColorInput::Primitive,
            0x04 => ColorInput::Shade,
            0x05 => ColorInput::Environment,
            0x06 => ColorInput::One,
            0x07 => ColorInput::Noise,
            0x08..=0x0f => ColorInput::Zero,
            _ => panic!("unexpected color combiner A value: 0x{:02x}", value),
        }
    }

    fn parse_b(value: u8) -> ColorInput {
        match value {
            0x00 => ColorInput::Combined,
            0x01 => ColorInput::Texel0,
            0x02 => ColorInput::Texel1,
            0x03 => ColorInput::Primitive,
            0x04 => ColorInput::Shade,
            0x05 => ColorInput::Environment,
            0x06 => ColorInput::Center,
            0x07 => ColorInput::K4,
            0x08..=0x0f => ColorInput::Zero,
            _ => panic!("unexpected color combiner B value: 0x{:02x}", value),
        }
    }

    fn parse_c(value: u8) -> ColorInput {
        match value {
            0x00 => ColorInput::Combined,
            0x01 => ColorInput::Texel0,
            0x02 => ColorInput::Texel1,
            0x03 => ColorInput::Primitive,
            0x04 => ColorInput::Shade,
            0x05 => ColorInput::Environment,
            0x06 => ColorInput::Scale,
            0x07 => ColorInput::CombinedAlpha,
            0x08 => ColorInput::Texel0Alpha,
            0x09 => ColorInput::Texel1Alpha,
            0x0a => ColorInput::PrimitiveAlpha,
            0x0b => ColorInput::ShadeAlpha,
            0x0c => ColorInput::EnvAlpha,
            0x0d => ColorInput::LodFraction,
            0x0e => ColorInput::PrimLodFrac,
            0x0f => ColorInput::K5,
            0x10..=0x1f => ColorInput::Zero,
            _ => panic!("unexpected color combiner C value: 0x{:02x}", value),
        }
    }

    fn parse_d(value: u8) -> ColorInput {
        match value {
            0x00 => ColorInput::Combined,
            0x01 => ColorInput::Texel0,
            0x02 => ColorInput::Texel1,
            0x03 => ColorInput::Primitive,
            0x04 => ColorInput::Shade,
            0x05 => ColorInput::Environment,
            0x06 => ColorInput::One,
            0x07 => ColorInput::Zero,
            _ => panic!("unexpected color combiner D value: 0x{:02x}", value),
        }
    }

    pub fn references(self) -> CombinerReference {
        match self {
            ColorInput::Combined => CombinerReference::COMBINED,
            ColorInput::Texel0 => CombinerReference::TEXEL_0,
            ColorInput::Texel1 => CombinerReference::TEXEL_1,
            ColorInput::Primitive => CombinerReference::PRIMITIVE,
            ColorInput::Shade => CombinerReference::SHADE,
            ColorInput::Environment => CombinerReference::ENVIRONMENT,
            ColorInput::One => CombinerReference::default(),
            ColorInput::Noise => CombinerReference::NOISE,
            ColorInput::Zero => CombinerReference::default(),
            ColorInput::Center => CombinerReference::CENTER,
            ColorInput::K4 => CombinerReference::K_4,
            ColorInput::Scale => CombinerReference::SCALE,
            ColorInput::CombinedAlpha => CombinerReference::COMBINED,
            ColorInput::Texel0Alpha => CombinerReference::TEXEL_0,
            ColorInput::Texel1Alpha => CombinerReference::TEXEL_1,
            ColorInput::PrimitiveAlpha => CombinerReference::PRIMITIVE,
            ColorInput::ShadeAlpha => CombinerReference::SHADE,
            ColorInput::EnvAlpha => CombinerReference::ENVIRONMENT,
            ColorInput::LodFraction => CombinerReference::LOD_FRACTION,
            ColorInput::PrimLodFrac => CombinerReference::PRIM_LOD_FRAC,
            ColorInput::K5 => CombinerReference::K_5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AlphaInput {
    Combined,
    Texel0,
    Texel1,
    Primitive,
    Shade,
    Environment,
    One,
    Zero,
    LodFraction,
    PrimLodFrac,
}

impl AlphaInput {
    fn parse_abd(value: u8) -> AlphaInput {
        match value {
            0x00 => AlphaInput::Combined,
            0x01 => AlphaInput::Texel0,
            0x02 => AlphaInput::Texel1,
            0x03 => AlphaInput::Primitive,
            0x04 => AlphaInput::Shade,
            0x05 => AlphaInput::Environment,
            0x06 => AlphaInput::One,
            0x07 => AlphaInput::Zero,
            _ => panic!("unexpected alpha combiner A/B/D value: 0x{:02x}", value),
        }
    }

    fn parse_c(value: u8) -> AlphaInput {
        match value {
            0x00 => AlphaInput::LodFraction,
            0x01 => AlphaInput::Texel0,
            0x02 => AlphaInput::Texel1,
            0x03 => AlphaInput::Primitive,
            0x04 => AlphaInput::Shade,
            0x05 => AlphaInput::Environment,
            0x06 => AlphaInput::PrimLodFrac,
            0x07 => AlphaInput::Zero,
            _ => panic!("unexpected alpha combiner C value: 0x{:02x}", value),
        }
    }

    pub fn references(self) -> CombinerReference {
        match self {
            AlphaInput::Combined => CombinerReference::COMBINED,
            AlphaInput::Texel0 => CombinerReference::TEXEL_0,
            AlphaInput::Texel1 => CombinerReference::TEXEL_1,
            AlphaInput::Primitive => CombinerReference::PRIMITIVE,
            AlphaInput::Shade => CombinerReference::SHADE,
            AlphaInput::Environment => CombinerReference::ENVIRONMENT,
            AlphaInput::One => CombinerReference::default(),
            AlphaInput::Zero => CombinerReference::default(),
            AlphaInput::LodFraction => CombinerReference::LOD_FRACTION,
            AlphaInput::PrimLodFrac => CombinerReference::PRIM_LOD_FRAC,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UnlitVertex<'a> {
    data: &'a [u8],
}

impl<'a> StructReader<'a> for UnlitVertex<'a> {
    const SIZE: usize = 16;
    fn new(data: &'a [u8]) -> UnlitVertex<'a> {
        UnlitVertex { data }
    }
}

impl<'a> UnlitVertex<'a> {
    pub fn position(self) -> [i16; 3] {
        [
            (&self.data[0x0..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0x2..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0x4..]).read_i16::<BigEndian>().unwrap(),
        ]
    }

    pub fn texcoord(self) -> [i16; 2] {
        [
            (&self.data[0x8..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0xa..]).read_i16::<BigEndian>().unwrap(),
        ]
    }

    pub fn color(self) -> [u8; 4] {
        [
            self.data[0xc],
            self.data[0xd],
            self.data[0xe],
            self.data[0xf],
        ]
    }
}

#[derive(Clone, Copy)]
pub struct LitVertex<'a> {
    data: &'a [u8],
}

impl<'a> StructReader<'a> for LitVertex<'a> {
    const SIZE: usize = 16;
    fn new(data: &'a [u8]) -> LitVertex<'a> {
        LitVertex { data }
    }
}

impl<'a> LitVertex<'a> {
    pub fn position(self) -> [i16; 3] {
        [
            (&self.data[0x0..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0x2..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0x4..]).read_i16::<BigEndian>().unwrap(),
        ]
    }

    pub fn texcoord(self) -> [i16; 2] {
        [
            (&self.data[0x8..]).read_i16::<BigEndian>().unwrap(),
            (&self.data[0xa..]).read_i16::<BigEndian>().unwrap(),
        ]
    }

    pub fn normal(self) -> [i8; 3] {
        [
            self.data[0xc] as i8,
            self.data[0xd] as i8,
            self.data[0xe] as i8,
        ]
    }

    pub fn alpha(self) -> u8 {
        self.data[0xf]
    }
}
