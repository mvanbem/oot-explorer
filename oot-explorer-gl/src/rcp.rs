use oot_explorer_core::fs::VromAddr;
use oot_explorer_core::gbi::{
    AlphaCombine, ColorCombine, GeometryMode, OtherModeH, OtherModeHMask, Qu0_16, Qu10_2, Qu1_11,
    TextureDepth, TextureFormat,
};
use std::ops::Range;
use thiserror::Error;

use crate::shader_state::{PaletteSource, ShaderState, TextureDescriptor};

#[derive(Debug)]
pub struct RcpState {
    pub vertex_slots: [Option<[u8; 20]>; 32],
    pub geometry_mode: GeometryMode,
    pub rdp_half_1: Option<u32>,
    pub rdp_other_mode: RdpOtherMode,
    pub combiner: Option<CombinerState>,
    pub texture_src: Option<TextureSource>,
    pub tiles: [Tile; 8],
    pub rsp_texture_state: RspTextureState,
    pub tmem: Tmem,
}

impl RcpState {
    pub fn shader_state(&self) -> ShaderState {
        let (texture_a, texture_b) = self.get_texture_state();
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
            texture_a,
            texture_b,
        }
    }

    fn get_texture_state(&self) -> (Option<TextureDescriptor>, Option<TextureDescriptor>) {
        // TODO: Support texture LOD.

        if !self.rsp_texture_state.enable {
            return (None, None);
        }

        let tile_a = self.get_tile_state(self.rsp_texture_state.tile);
        let tile_b = self.get_tile_state((self.rsp_texture_state.tile + 1) & 0x7);
        match self.rdp_other_mode.hi & OtherModeHMask::CYCLETYPE {
            OtherModeH::CYC_1CYCLE => (tile_a, None),
            OtherModeH::CYC_2CYCLE => (tile_a, tile_b),
            _ => unreachable!(),
        }
    }

    fn get_tile_state(&self, tile: u8) -> Option<TextureDescriptor> {
        let tile = &self.tiles[tile as usize];
        let dimensions = tile.dimensions.as_ref()?;
        let attributes = tile.attributes.as_ref()?;

        // TODO: Is this really the best way to bound texture tile size?
        let texels = ((dimensions.s.end.0 - dimensions.s.start.0) >> 2)
            * ((dimensions.t.end.0 - dimensions.t.start.0) >> 2);
        let tmem_words = texels / attributes.depth.texels_per_tmem_word::<u16>();
        let range = attributes.addr..attributes.addr + tmem_words;

        let source = self.tmem.get_source_for_address_range(range).unwrap();
        let palette_source = match attributes.format {
            TextureFormat::Ci => {
                let range = match attributes.depth {
                    TextureDepth::Bits4 => 256..272,
                    TextureDepth::Bits8 => 256..512,
                    x => unreachable!("there is no color-indexed format with depth {:?}", x),
                };
                let source = self.tmem.get_source_for_address_range(range).unwrap();
                match self.rdp_other_mode.hi & OtherModeHMask::TEXTLUT {
                    OtherModeH::TT_IA16 => PaletteSource::Ia(source),
                    // NOTE: Not comparing with TT_RGBA16 because it seems many display lists
                    // actually set TT_NONE. I don't know why this is supposed to work. Maybe the
                    // enable bit has no effect, so all that matters is the IA16 bit is clear?
                    _ => PaletteSource::Rgba(source),
                }
            }
            _ => PaletteSource::None,
        };
        Some(TextureDescriptor {
            source,
            palette_source,
            render_format: attributes.format,
            render_depth: attributes.depth,
            render_width: (((dimensions.s.end.0 - dimensions.s.start.0) >> 2) + 1) as usize,
            render_height: (((dimensions.t.end.0 - dimensions.t.start.0) >> 2) + 1) as usize,
            render_stride: attributes.stride as usize,
            render_palette: attributes.palette,
        })
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

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Tile {
    pub dimensions: Option<TileDimensions>,
    pub attributes: Option<TileAttributes>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TileDimensions {
    pub s: Range<Qu10_2>,
    pub t: Range<Qu10_2>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
pub struct RspTextureState {
    pub max_lod: u8,
    pub tile: u8,
    pub enable: bool,
    pub scale_s: Qu0_16,
    pub scale_t: Qu0_16,
}

/// Tracks the state of TMEM solely by reference to load operations.
///
/// This modeling is incomplete. Partially overwritten regions are discarded for simplicity on the
/// assumption that nobody uses TMEM that way. If this assumption is wrong, attempts to render from
/// affected tiles will warn about use of an uninitialized area in TMEM.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Tmem {
    regions: Vec<TmemRegion>,
}

impl Tmem {
    pub fn overlay(&mut self, new_region: TmemRegion) {
        let mut new_regions = vec![];
        let new_region_range = new_region.range.clone();
        let mut new_region = Some(new_region);
        for region in &self.regions {
            if region.range.end <= new_region_range.start {
                // This region is entirely before the new one.
                new_regions.push(region.clone());
            } else if new_region_range.end <= region.range.start {
                // This region is entirely after the new one.

                // Ensure the new region has been inserted.
                if let Some(new_region) = new_region.take() {
                    new_regions.push(new_region);
                }

                // Then copy this region over.
                new_regions.push(region.clone());
            } else {
                // This region partially or fully overlaps the new region. Just discard it.
            }
        }

        // Ensure the new region has been inserted.
        if let Some(new_region) = new_region {
            new_regions.push(new_region);
        }

        self.regions = new_regions;
    }

    pub fn get_source_for_address_range(
        &self,
        range: Range<u16>,
    ) -> Result<TmemSource, GetSourceError> {
        let mut sources = vec![];
        for region in &self.regions {
            if region.range.end <= range.start {
                // This region is entirely before the given range.
            } else if range.end <= region.range.start {
                // This region is entirely after the given range.
            } else {
                // This region partially or fully overlaps the given range.
                sources.push(&region.source);
            }
        }

        match sources.len() {
            0 => Ok(TmemSource::Undefined),
            1 => Ok(sources[0].clone()),
            _ => {
                eprintln!("WARNING: Texture references multiple TMEM regions. Support this!");
                Err(GetSourceError::MultipleSources)
            }
        }
    }
}

impl Default for Tmem {
    fn default() -> Self {
        Tmem { regions: vec![] }
    }
}

#[derive(Debug, Error)]
pub enum GetSourceError {
    #[error("region consists of multiple sources, unexpectedly")]
    MultipleSources,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TmemRegion {
    pub range: Range<u16>,
    pub source: TmemSource,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TmemSource {
    /// TMEM is considered undefined wherever there is no coverage by a region, but regions may
    /// themselves be explicitly undefined. The motivating example is a load with known destination
    /// and size, but invalid source.
    Undefined,
    LoadBlock {
        src_ptr: VromAddr,
        src_format: TextureFormat,
        src_depth: TextureDepth,
        load_dxt: Qu1_11,
        load_texels: u16,
        load_format: TextureFormat,
        load_depth: TextureDepth,
    },
    LoadTlut {
        ptr: VromAddr,
        count: u16,
    },
}

impl TmemSource {
    pub fn src(&self) -> Option<VromAddr> {
        match self {
            &TmemSource::Undefined => None,
            &TmemSource::LoadBlock { src_ptr, .. } => Some(src_ptr),
            &TmemSource::LoadTlut { ptr, .. } => Some(ptr),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cycle {
    Cycle1,
    Cycle2,
}
