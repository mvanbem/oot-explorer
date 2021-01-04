use oot_explorer_core::fs::VromAddr;
use oot_explorer_core::gbi::{
    AlphaCombine, ColorCombine, CombinerReference, GeometryMode, OtherModeH, OtherModeHMask,
    Qu0_16, Qu10_2, Qu1_11, TextureDepth, TextureFormat,
};
use std::ops::Range;
use thiserror::Error;

use crate::shader_state::{
    PaletteSource, ShaderState, TexCoordParams, TextureDescriptor, TextureParams, TextureState,
};

#[derive(Debug)]
pub struct RcpState {
    pub vertex_slots: [Option<[u8; 20]>; 32],
    pub geometry_mode: GeometryMode,
    pub rdp_half_1: Option<u32>,
    pub rdp_other_mode: RdpOtherMode,
    pub primitive_color: Option<[u8; 4]>,
    pub env_color: Option<[u8; 4]>,
    pub prim_lod_frac: Option<u8>,
    pub combiner: Option<CombinerState>,
    pub texture_src: Option<TextureSource>,
    pub tiles: [Tile; 8],
    pub rsp_texture_state: RspTextureState,
    pub tmem: Tmem,
}

impl RcpState {
    pub fn to_shader_state(&self) -> ShaderState {
        let (texture_0, texture_1) = self.get_texture_state();
        ShaderState {
            two_cycle_mode: match self.rdp_other_mode.hi & OtherModeHMask::CYCLETYPE {
                x if x == OtherModeH::CYC_1CYCLE => false,
                x if x == OtherModeH::CYC_2CYCLE => true,
                _ => panic!(
                    "display list did not choose one- or two-cycle mode: {:#?}",
                    self
                ),
            },
            primitive_color: self.primitive_color,
            env_color: self.env_color,
            prim_lod_frac: self.prim_lod_frac,
            combiner: self.combiner.as_ref().unwrap().clone(),
            texture_0,
            texture_1,
        }
    }

    fn get_texture_state(&self) -> (Option<TextureState>, Option<TextureState>) {
        let references = self.combiner.as_ref().unwrap().references();

        if !self.rsp_texture_state.enable {
            return (None, None);
        }

        // LOD is not implemented.
        assert_eq!(
            self.rdp_other_mode.hi & OtherModeHMask::TEXTLOD,
            OtherModeH::TL_TILE,
        );

        match self.rdp_other_mode.hi & OtherModeHMask::TEXTDETAIL {
            OtherModeH::TD_CLAMP => (),
            OtherModeH::TD_SHARPEN => unimplemented!("texture sharpening"),
            OtherModeH::TD_DETAIL => unimplemented!("detail texture"),
            _ => unreachable!(),
        }

        let tile_0 = if references.test(CombinerReference::TEXEL_0) {
            self.get_tile_state(self.rsp_texture_state.tile)
        } else {
            None
        };
        let tile_1 = if references.test(CombinerReference::TEXEL_1) {
            self.get_tile_state((self.rsp_texture_state.tile + 1) & 0x7)
        } else {
            None
        };
        (tile_0, tile_1)
    }

    fn get_tile_state(&self, tile: u8) -> Option<TextureState> {
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

        let calc_size = |bounds: &Range<Qu10_2>, mask: u8| {
            // Calculate the full range of tile coordinates that might be sampled, not considering
            // masking. Shift right two places because the tile coordinates are in 10.2 fixed point.
            // Add one because the tile coordinates are inclusive bounds.
            let size = ((bounds.end.0 - bounds.start.0) >> 2) + 1;

            if mask == 0 {
                // No masking, so the full range might be sampled.
                size
            } else {
                // Masking exposes only the N least significant bits of the integer tile coordinate.
                size.min(1 << mask)
            }
        };
        let width = calc_size(&dimensions.s, attributes.mask_s);
        let height = calc_size(&dimensions.t, attributes.mask_t);

        Some(TextureState {
            descriptor: TextureDescriptor {
                source,
                palette_source,
                render_format: attributes.format,
                render_depth: attributes.depth,
                render_width: width as usize,
                render_height: height as usize,
                render_stride: attributes.stride as usize,
                render_palette: attributes.palette,
            },
            params: TextureParams {
                s: TexCoordParams {
                    range: dimensions.s.clone(),
                    mirror: attributes.mirror_s,
                    mask: attributes.mask_s,
                    shift: attributes.shift_s,
                    clamp: attributes.clamp_s,
                },
                t: TexCoordParams {
                    range: dimensions.t.clone(),
                    mirror: attributes.mirror_t,
                    mask: attributes.mask_t,
                    shift: attributes.shift_t,
                    clamp: attributes.clamp_t,
                },
            },
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

impl CombinerState {
    pub fn references(&self) -> CombinerReference {
        self.color_0.references()
            | self.alpha_0.references()
            | self.color_1.references()
            | self.alpha_1.references()
    }
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
/// affected tiles will warn about undefined contents.
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
