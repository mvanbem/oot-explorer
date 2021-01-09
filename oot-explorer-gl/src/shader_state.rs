use oot_explorer_core::gbi::{CombinerReference, Qu10_2, TextureDepth, TextureFormat};
use oot_explorer_expr as expr;
use std::fmt::Write;
use std::ops::Range;

use crate::glsl_float_constant::GlslFloatConstant;
use crate::glsl_vec3_constant::GlslVec3Constant;
use crate::rcp::{CombinerState, Cycle, TmemSource};
use crate::to_expr::ToExpr;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ShaderState {
    pub two_cycle_mode: bool,
    pub primitive_color: Option<[u8; 4]>,
    pub env_color: Option<[u8; 4]>,
    pub prim_lod_frac: Option<u8>,
    pub combiner: CombinerState,
    pub texture_0: Option<TextureState>,
    pub texture_1: Option<TextureState>,
    pub z_upd: bool,
    pub decal: bool,
}

impl ShaderState {
    fn color_0_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>) -> expr::Key {
        self.combiner.color_0.to_expr(ctx, Cycle::Cycle1)
    }

    fn alpha_0_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>) -> expr::Key {
        self.combiner.alpha_0.to_expr(ctx, Cycle::Cycle1)
    }

    fn color_1_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>) -> expr::Key {
        self.combiner.color_1.to_expr(ctx, Cycle::Cycle2)
    }

    fn alpha_1_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>) -> expr::Key {
        self.combiner.alpha_1.to_expr(ctx, Cycle::Cycle2)
    }

    pub fn to_glsl(&self) -> String {
        let mut color_ctx = expr::Context::new();
        let mut alpha_ctx = expr::Context::new();
        let color_0 = self.color_0_expr(&mut color_ctx);
        let alpha_0 = self.alpha_0_expr(&mut alpha_ctx);
        let color_1 = self.color_1_expr(&mut color_ctx);
        let alpha_1 = self.alpha_1_expr(&mut alpha_ctx);
        assert!(self.two_cycle_mode);

        let references = self.combiner.references();

        let mut glsl = String::new();
        write!(
            glsl,
            r#"#version 300 es
precision highp float;
precision highp int;
"#,
        )
        .unwrap();

        if references.test(CombinerReference::TEXEL_0) && self.texture_0.is_some() {
            write!(glsl, "uniform sampler2D u_texture0;\n").unwrap();
        }
        if references.test(CombinerReference::TEXEL_1) && self.texture_1.is_some() {
            write!(glsl, "uniform sampler2D u_texture1;\n").unwrap();
        }
        if references.test(CombinerReference::SHADE) {
            write!(glsl, "in vec4 v_shade;\n").unwrap();
        }
        if references.test(CombinerReference::TEXEL_0)
            || references.test(CombinerReference::TEXEL_1)
        {
            write!(glsl, "in vec2 v_texCoord;\n").unwrap();
        }

        write!(glsl, "layout(location = 0) out vec4 fragColor;\n").unwrap();
        write!(
            glsl,
            "struct ProcessedTexCoord {{ ivec2 texel; vec2 fract; }};\n",
        )
        .unwrap();

        if references.test(CombinerReference::TEXEL_0) {
            if let Some(ref texture) = self.texture_0 {
                ShaderState::write_process_tex_coord_fn(&mut glsl, texture, '0');
            }
        }
        if references.test(CombinerReference::TEXEL_1) {
            if let Some(ref texture) = self.texture_1 {
                ShaderState::write_process_tex_coord_fn(&mut glsl, texture, '1');
            }
        }

        write!(
            glsl,
            r#"void main() {{
"#,
        )
        .unwrap();

        if references.test(CombinerReference::PRIMITIVE) {
            // TODO: Determine an appropriate default primitive color.
            let prim = self.primitive_color.unwrap_or([128; 4]);
            write!(
                glsl,
                "vec4 prim = vec4({}.0, {}.0, {}.0, {}.0) / 255.0;\n",
                prim[0], prim[1], prim[2], prim[3],
            )
            .unwrap();
        }
        if references.test(CombinerReference::ENVIRONMENT) {
            // TODO: Determine an appropriate default environment color.
            let env = self.env_color.unwrap_or([128; 4]);
            write!(
                glsl,
                "vec4 env = vec4({}.0, {}.0, {}.0, {}.0) / 255.0;\n",
                env[0], env[1], env[2], env[3],
            )
            .unwrap();
        }
        if references.test(CombinerReference::LOD_FRACTION) {
            unimplemented!("LOD is not implemented");
        }
        if references.test(CombinerReference::PRIM_LOD_FRAC) {
            let prim_lod_frac = self.prim_lod_frac.expect("undefined prim_lod_frac");
            write!(glsl, "float primLodFrac = {}.0 / 255.0;\n", prim_lod_frac,).unwrap();
        }

        // Texturing.
        if (references.test(CombinerReference::TEXEL_0) && self.texture_0.is_some())
            || (references.test(CombinerReference::TEXEL_1) && self.texture_1.is_some())
        {
            write!(glsl, "ivec2 texCoord = ivec2(round(v_texCoord));\n").unwrap();
        }
        if references.test(CombinerReference::TEXEL_0) {
            ShaderState::write_glsl_for_texture(&mut glsl, &self.texture_0, '0');
        }
        if references.test(CombinerReference::TEXEL_1) {
            ShaderState::write_glsl_for_texture(&mut glsl, &self.texture_1, '1');
        }

        // Emit the two-cycle color combiner expressions.
        if references.test(CombinerReference::COMBINED) {
            write!(
                glsl,
                "vec4 combined = vec4({:?}, {:?});\n",
                color_ctx.get_with_ctx(color_0).unwrap(),
                alpha_ctx.get_with_ctx(alpha_0).unwrap(),
            )
            .unwrap();
        }
        write!(
            glsl,
            r#"fragColor = vec4({:?}, {:?});
if (fragColor.a == 0.0) {{
  discard;
}}
}}
"#,
            color_ctx.get_with_ctx(color_1).unwrap(),
            alpha_ctx.get_with_ctx(alpha_1).unwrap(),
        )
        .unwrap();

        glsl
    }

    fn write_process_tex_coord_fn(glsl: &mut String, texture: &TextureState, index: char) {
        // NOTE: All texture coordinate GLSL variables are in signed fixed-point with five
        // fractional bits. Sample coordinates are just plain integers.

        write!(
            glsl,
            "ProcessedTexCoord processTexCoord{index}(ivec2 texCoord, ivec2 offset) {{\n",
            index = index,
        )
        .unwrap();

        // Apply shift.
        //
        // TODO: Left shifts might need to be masked to some particular width. Not sure what's
        // supposed to happen in that case.
        let shift_expr = |params: &TexCoordParams| match params.shift & 0xf {
            0 => "",
            1 => " >> 1",
            2 => " >> 2",
            3 => " >> 3",
            4 => " >> 4",
            5 => " >> 5",
            6 => " >> 6",
            7 => " >> 7",
            8 => " >> 8",
            9 => " >> 9",
            10 => " >> 10",
            11 => " << 5",
            12 => " << 4",
            13 => " << 3",
            14 => " << 2",
            15 => " << 1",
            _ => unreachable!(),
        };
        write!(
            glsl,
            "ivec2 shifted = ivec2(texCoord.s{shift_s}, texCoord.t{shift_t}) + offset;\n",
            shift_s = shift_expr(&texture.params.s),
            shift_t = shift_expr(&texture.params.t),
        )
        .unwrap();

        // Align the incoming texture coordinate, in texture image coordinate space, to the
        // min-corner of tile space.
        write!(
            glsl,
            "ivec2 aligned = shifted - ivec2({min_s}, {min_t});\n",
            // Shift left to convert from Qu10_2 to Qi26_5.
            min_s = texture.params.s.range.start.0 << 3,
            min_t = texture.params.t.range.start.0 << 3,
        )
        .unwrap();

        // Apply clamping if enabled. Note that clamping is always enabled when not masking,
        // otherwise clamping is controlled by its flag.
        write!(glsl, "ivec2 clamped = ivec2(").unwrap();
        let write_clamp_component = |glsl: &mut String, component, params: &TexCoordParams| {
            if params.mask == 0 || params.clamp {
                // Clamping. Clamp the aligned coordinate
                // Shift left to convert from Qu10_2 to Qi26_5.
                let span = (params.range.end.0 - params.range.start.0) << 3;
                write!(
                    glsl,
                    "clamp(aligned.{component}, -{neg_min}, {max})",
                    component = component,
                    neg_min = if params.mirror { span } else { 0 },
                    max = span,
                )
                .unwrap();
            } else {
                // Not clamping. Just pass through the aligned coordinate.
                write!(glsl, "aligned.{component}", component = component,).unwrap();
            }
        };
        write_clamp_component(glsl, 's', &texture.params.s);
        write!(glsl, ", ").unwrap();
        write_clamp_component(glsl, 't', &texture.params.t);
        write!(glsl, ");\n").unwrap();

        // Apply masking.
        write!(glsl, "ivec2 masked = clamped & ivec2(",).unwrap();
        let write_mask_component = |glsl: &mut String, params: &TexCoordParams| {
            write!(
                glsl,
                "{mask}",
                mask = if params.mask > 0 {
                    (1 << (params.mask as i32 + params.mirror as i32 + 5)) - 1
                } else {
                    -1
                },
            )
            .unwrap();
        };
        write_mask_component(glsl, &texture.params.s);
        write!(glsl, ", ").unwrap();
        write_mask_component(glsl, &texture.params.t);
        write!(glsl, ");\n").unwrap();

        // Extract the integer texel coordinate.
        write!(glsl, "ivec2 texelCoord = masked >> 5u;\n").unwrap();

        // Apply wrapping.
        let write_wrap_component = |glsl: &mut String, component, params: &TexCoordParams| {
            let mirror_bit = 1 << params.mask;
            let subtract_from = mirror_bit - 1;
            write!(
                glsl,
                "texelCoord.{component} = {subtract_from} - abs({subtract_from} - texelCoord.{component} + ((texelCoord.{component} & {mirror_bit}) >> {mirror_shift}));\n",
                component = component,
                mirror_bit = mirror_bit,
                subtract_from = subtract_from,
                mirror_shift = params.mask,
            )
            .unwrap();
        };
        write_wrap_component(glsl, 's', &texture.params.s);
        write_wrap_component(glsl, 't', &texture.params.t);

        write!(
            glsl,
            r#"texelCoord = clamp(texelCoord, ivec2(0), ivec2({max_s}, {max_t}));
vec2 texelFract = vec2(masked & 0x1f) / 32.0;
return ProcessedTexCoord(texelCoord, texelFract);
}}
"#,
            max_s =
                if texture.params.s.mirror { 2 } else { 1 } * texture.descriptor.render_width - 1,
            max_t =
                if texture.params.t.mirror { 2 } else { 1 } * texture.descriptor.render_height - 1,
        )
        .unwrap();
    }

    fn write_glsl_for_texture(glsl: &mut String, texture: &Option<TextureState>, index: char) {
        if let Some(ref _texture) = texture {
            write!(
                glsl,
                r#"
ProcessedTexCoord coord{index} = processTexCoord{index}(texCoord, ivec2(0, 0));
vec2 fract{index} = coord{index}.fract;
ivec2 coordTL{index} = coord{index}.texel;
ivec2 coordTR{index} = processTexCoord{index}(texCoord, ivec2(32, 0)).texel;
ivec2 coordBL{index} = processTexCoord{index}(texCoord, ivec2(0, 32)).texel;
ivec2 coordBR{index} = processTexCoord{index}(texCoord, ivec2(32, 32)).texel;
vec4 sampleTL{index} = texelFetch(u_texture{index}, coordTL{index}, 0);
vec4 sampleTR{index} = texelFetch(u_texture{index}, coordTR{index}, 0);
vec4 sampleBL{index} = texelFetch(u_texture{index}, coordBL{index}, 0);
vec4 sampleBR{index} = texelFetch(u_texture{index}, coordBR{index}, 0);
vec4 texel{index};
if (fract{index}.s < (1.0 - fract{index}.t)) {{
vec4 ds = sampleTR{index} - sampleTL{index};
vec4 dt = sampleBL{index} - sampleTL{index};
texel{index} = sampleTL{index} + fract{index}.s * ds + fract{index}.t * dt;
}} else {{
vec4 ds = sampleBL{index} - sampleBR{index};
vec4 dt = sampleTR{index} - sampleBR{index};
texel{index} = sampleBR{index} + (1.0 - fract{index}.s) * ds + (1.0 - fract{index}.t) * dt;
}}"#,
                index = index,
            )
            .unwrap();
        } else {
            write!(
                glsl,
                "vec4 texel{index} = vec4(1.0, 0.0, 1.0, 0.5);\n",
                index = index
            )
            .unwrap();
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureState {
    pub descriptor: TextureDescriptor,
    pub params: TextureParams,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureParams {
    pub s: TexCoordParams,
    pub t: TexCoordParams,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TexCoordParams {
    pub range: Range<Qu10_2>,
    pub mirror: bool,
    pub mask: u8,
    pub shift: u8,
    pub clamp: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureDescriptor {
    pub source: TmemSource,
    pub palette_source: PaletteSource,
    pub render_format: TextureFormat,
    pub render_depth: TextureDepth,
    pub render_width: usize,
    pub render_height: usize,
    pub render_stride: usize,
    pub render_palette: u8,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PaletteSource {
    None,
    Rgba(TmemSource),
    Ia(TmemSource),
}
