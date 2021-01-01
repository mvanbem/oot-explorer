use oot_explorer_core::fs::VromAddr;
use oot_explorer_core::gbi::{
    AlphaCombine, ColorCombine, GeometryMode, OtherModeH, OtherModeHMask, Qu0_16, Qu1_11,
    TextureDepth, TextureFormat,
};

use crate::shader_state::ShaderState;

#[derive(Debug)]
pub struct RcpState {
    pub vertex_slots: [Option<[u8; 20]>; 32],
    pub geometry_mode: GeometryMode,
    pub rdp_half_1: Option<u32>,
    pub rdp_other_mode: RdpOtherMode,
    pub combiner: Option<CombinerState>,
    pub texture_src: Option<TextureSource>,
    pub tiles: [Tile; 8],
    pub tmem: Tmem,
}
impl RcpState {
    pub fn shader_state(&self) -> ShaderState {
        ShaderState {
            two_cycle_mode: match self.rdp_other_mode.hi & OtherModeHMask::CYCLETYPE {
                x if x == OtherModeH::CYC_1CYCLE => false,
                x if x == OtherModeH::CYC_2CYCLE => true,
                _ => panic!(
                    "display list did not choose one- or two-cycle mode: {:#?}",
                    self
                ),
            },
            combiner: self.combiner.as_ref().unwrap().clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RdpOtherMode {
    pub hi: OtherModeH,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CombinerState {
    pub color_0: ColorCombine,
    pub alpha_0: AlphaCombine,
    pub color_1: ColorCombine,
    pub alpha_1: AlphaCombine,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureSource {
    pub format: TextureFormat,
    pub depth: TextureDepth,
    pub ptr: VromAddr,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Tile {
    pub dimensions: Option<TileDimensions>,
    pub attributes: Option<TileAttributes>,
    pub mip_scale: Option<TileMipScale>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileDimensions {
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileAttributes {
    pub format: TextureFormat,
    pub depth: TextureDepth,
    pub stride: u16,
    pub addr: u16,
    pub palette: u8,
    pub clamp_t: bool,
    pub mirror_t: bool,
    pub mask_t: u8,
    pub shift_t: u8,
    pub clamp_s: bool,
    pub mirror_s: bool,
    pub mask_s: u8,
    pub shift_s: u8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileMipScale {
    pub level: u8,
    pub enable: bool,
    pub scale_s: Qu0_16,
    pub scale_t: Qu0_16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tmem {
    Undefined,
    LoadBlock {
        dxt: Qu1_11,
        ptr: VromAddr,
        len: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cycle {
    Cycle1,
    Cycle2,
}
