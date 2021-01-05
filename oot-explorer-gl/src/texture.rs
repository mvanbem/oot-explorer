use std::ops::Range;

use oot_explorer_core::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use oot_explorer_core::gbi::{Qu1_11, TextureDepth, TextureFormat};
use scoped_owner::ScopedOwner;
use thiserror::Error;

use crate::rcp::TmemSource;
use crate::shader_state::{PaletteSource, TextureDescriptor};

/// Applies both layers of TMEM word swapping. One is performed by the LoadBlock command based on
/// load_dxt and the word offset. The other is performed by the RDP based on y. These two swaps may
/// cancel out.
fn word_swap(offset: usize, load_dxt: Qu1_11, render_y: usize) -> usize {
    let load_line = ((offset / 8) * (load_dxt.0 as usize)) >> 11;
    let load_swap = load_line & 1 == 1;

    let render_swap = render_y & 1 == 1;

    if load_swap != render_swap {
        offset ^ 0x4
    } else {
        offset
    }
}

fn rgb5a1_to_rgba8(x: u16) -> [u8; 4] {
    let expand_5_to_8 = |x| (x << 3) | (x >> 2);

    let r = expand_5_to_8(((x >> 11) & 0x1f) as u8);
    let g = expand_5_to_8(((x >> 6) & 0x1f) as u8);
    let b = expand_5_to_8(((x >> 1) & 0x1f) as u8);
    let a = if x & 0x01 == 0x01 { 0xff } else { 0x00 };
    [r, g, b, a]
}

trait PaletteDecoder {
    fn decode(&self, x: u16) -> [u8; 4];
}

struct RgbaPaletteDecoder;

impl PaletteDecoder for RgbaPaletteDecoder {
    fn decode(&self, x: u16) -> [u8; 4] {
        rgb5a1_to_rgba8(x)
    }
}

struct IaPaletteDecoder;

impl PaletteDecoder for IaPaletteDecoder {
    fn decode(&self, x: u16) -> [u8; 4] {
        let i = (x >> 8) as u8;
        let a = x as u8;
        [i, i, i, a]
    }
}

/// A decoded texture.
#[derive(Clone)]
pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    /// RGBA8 format. Row-major from top to bottom and left to right. No padding.
    pub data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("texels were loaded via LoadTlut")]
    UnexpectedTexelSource,

    #[error("texels were accessed from a TMEM region with undefined contents")]
    UndefinedTexels,

    #[error("too few texels for the given tile parameters: want {want}, but got {got}")]
    UnderflowedTexels { want: u32, got: u32 },

    #[error("inaccessible texels: {0:?}")]
    InaccessibleTexels(VirtualSliceError),

    #[error("palette was loaded via LoadBlock")]
    UnexpectedPaletteSource,

    #[error("palette was accessed from a TMEM region with undefined contents")]
    UndefinedPalette,

    #[error("inaccessible palette: {0:?}")]
    InaccessiblePalette(VirtualSliceError),
}

fn get_texture_source_and_load_information(
    texture: &TextureDescriptor,
) -> Result<
    (
        VromAddr,
        TextureFormat,
        TextureDepth,
        Qu1_11,
        TextureFormat,
        TextureDepth,
        u32,
    ),
    DecodeError,
> {
    match texture.source {
        TmemSource::LoadBlock {
            src_ptr,
            src_format,
            src_depth,
            load_dxt,
            load_texels,
            load_format,
            load_depth,
            ..
        } => Ok((
            src_ptr,
            src_format,
            src_depth,
            load_dxt,
            load_format,
            load_depth,
            (8 * load_texels as u32) / src_depth.texels_per_tmem_word::<u32>(),
        )),
        TmemSource::LoadTlut { .. } => Err(DecodeError::UnexpectedTexelSource),
        TmemSource::Undefined => Err(DecodeError::UndefinedTexels),
    }
}

pub fn get_texel_data<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    texture: &TextureDescriptor,
    src_ptr: VromAddr,
    load_len: u32,
) -> Result<(&'a [u8], usize), DecodeError> {
    let expected_len = (8 * texture.render_width
        / texture.render_depth.texels_per_tmem_word::<usize>()
        + 8 * (texture.render_height - 1) * texture.render_stride) as u32;
    if load_len < expected_len {
        return Err(DecodeError::UnderflowedTexels {
            want: expected_len,
            got: load_len,
        });
    }

    let src = fs
        .get_virtual_slice(scope, src_ptr..src_ptr + expected_len)
        .map_err(|e| DecodeError::InaccessibleTexels(e))?;

    let stride_bytes = 8 * texture.render_stride;

    Ok((src, stride_bytes))
}

pub fn get_palette_data<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    texture: &TextureDescriptor,
    entry_range: Range<u32>,
) -> Result<&'a [u8], DecodeError> {
    let source = match texture.palette_source {
        PaletteSource::None => {
            unreachable!("BUG: this should always be set for color-indexed formats")
        }
        PaletteSource::Rgba(ref source) => Ok(source),
        PaletteSource::Ia(ref source) => Ok(source),
    }?;
    match source {
        &TmemSource::LoadBlock { .. } => Err(DecodeError::UnexpectedPaletteSource),
        &TmemSource::LoadTlut { ptr, count } => {
            assert!(count as u32 >= entry_range.end);
            fs.get_virtual_slice(
                scope,
                (ptr + 2 * entry_range.start)..(ptr + 2 * entry_range.end),
            )
            .map_err(|e| DecodeError::InaccessiblePalette(e))
        }
        &TmemSource::Undefined => Err(DecodeError::UndefinedPalette),
    }
}

trait TexelDecoder {
    fn decode<'a>(
        &self,
        scope: &'a ScopedOwner,
        fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError>;
}

/// Decodes texels from the color-indexed 4-bit format.
///
/// Two texels pack into one byte:
///
/// - `7:4` 4-bit color index for first texel
/// - `3:0` 4-bit color index for second texel
struct Ci4TexelDecoder<P: PaletteDecoder>(P);

impl<P: PaletteDecoder> TexelDecoder for Ci4TexelDecoder<P> {
    fn decode<'a>(
        &self,
        scope: &'a ScopedOwner,
        fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        let palette_base = 16 * texture.render_palette as u32;
        let palette = get_palette_data(scope, fs, texture, palette_base..palette_base + 16)?;
        for y in 0..texture.render_height {
            for x in (0..texture.render_width).step_by(2) {
                let index_offset = word_swap(stride_bytes * y + x / 2, load_dxt, y);
                let indexes = src[index_offset] as usize;
                let color1_offset = 2 * (indexes >> 4);
                let color1 =
                    ((palette[color1_offset] as u16) << 8) | palette[color1_offset + 1] as u16;
                let color2_offset = 2 * (indexes & 0x0f);
                let color2 =
                    ((palette[color2_offset] as u16) << 8) | palette[color2_offset + 1] as u16;
                dst.extend_from_slice(&self.0.decode(color1));
                dst.extend_from_slice(&self.0.decode(color2));
            }
        }
        Ok(())
    }
}

/// Decodes texels from the intensity-alpha 4-bit format.
///
/// Two texels pack into one byte:
///
/// - `7:5` 3-bit intensity for first texel
/// - `4` 1-bit alpha for first texel
/// - `3:1` 3-bit intensity for second texel
/// - `0` 1-bit alpha for second texel
struct Ia4TexelDecoder;

impl TexelDecoder for Ia4TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in (0..texture.render_width).step_by(2) {
                let offset = word_swap(stride_bytes * y + x / 2, load_dxt, y);
                let x = src[offset];
                let i1 = (x & 0xe0) | ((x >> 3) & 0x8c) | ((x >> 6) & 0x03);
                let a1 = if x & 0x10 == 0x10 { 0xff } else { 0x00 };
                let i2 = ((x << 4) & 0xe0) | ((x << 1) & 0x8c) | ((x >> 2) & 0x03);
                let a2 = if x & 0x01 == 0x01 { 0xff } else { 0x00 };
                dst.extend_from_slice(&[i1, i1, i1, a1, i2, i2, i2, a2]);
            }
        }
        Ok(())
    }
}

/// Decodes texels from the intensity 4-bit format.
///
/// Two texels pack into one byte:
///
/// - `7:4` 4-bit intensity for first texel
/// - `3:0` 4-bit intensity for second texel
struct I4TexelDecoder;

impl TexelDecoder for I4TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in (0..texture.render_width).step_by(2) {
                let offset = word_swap(stride_bytes * y + x / 2, load_dxt, y);
                let x = src[offset];
                let i1 = (x & 0xf0) | ((x >> 4) & 0x0f);
                let i2 = ((x << 4) & 0xf0) | (x & 0x0f);
                dst.extend_from_slice(&[i1, i1, i1, i1, i2, i2, i2, i2]);
            }
        }
        Ok(())
    }
}

/// Decodes texels from the color-indexed 8-bit format.
///
/// Each texel packs into one byte:
///
/// - `7:0` 8-bit color index
struct Ci8TexelDecoder<P: PaletteDecoder>(P);

impl<P: PaletteDecoder> TexelDecoder for Ci8TexelDecoder<P> {
    fn decode<'a>(
        &self,
        scope: &'a ScopedOwner,
        fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        let palette = get_palette_data(scope, fs, texture, 0..256)?;
        for y in 0..texture.render_height {
            for x in 0..texture.render_width {
                let index_offset = word_swap(stride_bytes * y + x, load_dxt, y);
                let color_offset = 2 * src[index_offset] as usize;
                let color =
                    ((palette[color_offset] as u16) << 8) | palette[color_offset + 1] as u16;
                dst.extend_from_slice(&self.0.decode(color));
            }
        }
        Ok(())
    }
}

/// Decodes texels from the intensity-alpha 8-bit format.
///
/// Each texel packs into one byte:
///
/// - `7:4` 4-bit intensity
/// - `3:0` 4-bit alpha
struct Ia8TexelDecoder;

impl TexelDecoder for Ia8TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in 0..texture.render_width {
                let offset = word_swap(stride_bytes * y + x, load_dxt, y);
                let x = src[offset];
                let i = (x & 0xf0) | ((x >> 4) & 0x0f);
                let a = ((x << 4) & 0xf0) | (x & 0x0f);
                dst.extend_from_slice(&[i, i, i, a]);
            }
        }
        Ok(())
    }
}

/// Decodes texels from the intensity 8-bit format.
///
/// Each texel packs into one byte:
///
/// - `7:0` 8-bit intensity
struct I8TexelDecoder;

impl TexelDecoder for I8TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in 0..texture.render_width {
                let offset = word_swap(stride_bytes * y + x, load_dxt, y);
                let i = src[offset];
                dst.extend_from_slice(&[i, i, i, i]);
            }
        }
        Ok(())
    }
}

/// Decodes texels from the RGBA 16-bit format.
///
/// Each texel packs into two big-endian bytes:
///
/// - `15:11` 5-bit red
/// - `10:6` 5-bit green
/// - `5:1` 5-bit blue
/// - `0` 1-bit alpha
struct Rgba16TexelDecoder;

impl TexelDecoder for Rgba16TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in 0..texture.render_width {
                let offset = word_swap(stride_bytes * y + 2 * x, load_dxt, y);
                let x = ((src[offset] as u16) << 8) | src[offset + 1] as u16;
                dst.extend_from_slice(&rgb5a1_to_rgba8(x));
            }
        }
        Ok(())
    }
}

/// Decodes texels from the intensity-alpha 16-bit format.
///
/// Each texel packs into two big-endian bytes:
///
/// - `15:8` 8-bit intensity
//  - `7:0` 8-bit alpha
struct Ia16TexelDecoder;

impl TexelDecoder for Ia16TexelDecoder {
    fn decode<'a>(
        &self,
        _scope: &'a ScopedOwner,
        _fs: &mut LazyFileSystem<'a>,
        texture: &TextureDescriptor,
        src: &[u8],
        dst: &mut Vec<u8>,
        stride_bytes: usize,
        load_dxt: Qu1_11,
    ) -> Result<(), DecodeError> {
        for y in 0..texture.render_height {
            for x in 0..texture.render_width {
                let offset = word_swap(stride_bytes * y + 2 * x, load_dxt, y);
                let i = src[offset];
                let a = src[offset + 1];
                dst.extend_from_slice(&[i, i, i, a]);
            }
        }
        Ok(())
    }
}

pub fn decode<'a>(
    scope: &'a ScopedOwner,
    fs: &mut LazyFileSystem<'a>,
    texture: &TextureDescriptor,
) -> Result<DecodedTexture, DecodeError> {
    let (src_ptr, src_format, src_depth, load_dxt, load_format, load_depth, load_len) =
        get_texture_source_and_load_information(texture)?;

    // Format conversion during load is not implemented.
    assert_eq!(src_format, load_format);
    assert_eq!(src_depth, load_depth);

    let rgba16: &dyn TexelDecoder = &Rgba16TexelDecoder;
    let ci4_rgba: &dyn TexelDecoder = &Ci4TexelDecoder(RgbaPaletteDecoder);
    let ci4_ia: &dyn TexelDecoder = &Ci4TexelDecoder(IaPaletteDecoder);
    let ci8_rgba: &dyn TexelDecoder = &Ci8TexelDecoder(RgbaPaletteDecoder);
    let ci8_ia: &dyn TexelDecoder = &Ci8TexelDecoder(IaPaletteDecoder);
    let ia4: &dyn TexelDecoder = &Ia4TexelDecoder;
    let ia8: &dyn TexelDecoder = &Ia8TexelDecoder;
    let ia16: &dyn TexelDecoder = &Ia16TexelDecoder;
    let i4: &dyn TexelDecoder = &I4TexelDecoder;
    let i8: &dyn TexelDecoder = &I8TexelDecoder;

    let (src, stride_bytes) = get_texel_data(scope, fs, texture, src_ptr, load_len)?;
    let mut dst = Vec::with_capacity(4 * texture.render_width * texture.render_height);

    (match (
        texture.render_format,
        texture.render_depth,
        &texture.palette_source,
    ) {
        (TextureFormat::Rgba, TextureDepth::Bits16, _) => rgba16,
        (TextureFormat::Ci, TextureDepth::Bits4, PaletteSource::Rgba(_)) => ci4_rgba,
        (TextureFormat::Ci, TextureDepth::Bits4, PaletteSource::Ia(_)) => ci4_ia,
        (TextureFormat::Ci, TextureDepth::Bits8, PaletteSource::Rgba(_)) => ci8_rgba,
        (TextureFormat::Ci, TextureDepth::Bits8, PaletteSource::Ia(_)) => ci8_ia,
        (TextureFormat::Ia, TextureDepth::Bits4, _) => ia4,
        (TextureFormat::Ia, TextureDepth::Bits8, _) => ia8,
        (TextureFormat::Ia, TextureDepth::Bits16, _) => ia16,
        (TextureFormat::I, TextureDepth::Bits4, _) => i4,
        (TextureFormat::I, TextureDepth::Bits8, _) => i8,
        x => {
            panic!("unimplemented format: {:?}", x);
        }
    })
    .decode(scope, fs, texture, src, &mut dst, stride_bytes, load_dxt)?;

    Ok(DecodedTexture {
        width: texture.render_width,
        height: texture.render_height,
        data: dst,
    })
}
