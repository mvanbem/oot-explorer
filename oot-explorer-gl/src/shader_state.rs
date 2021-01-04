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
            write!(glsl, "uniform vec2 u_texture0InvSize;\n").unwrap();
        }
        if references.test(CombinerReference::TEXEL_1) && self.texture_1.is_some() {
            write!(glsl, "uniform sampler2D u_texture1;\n").unwrap();
            write!(glsl, "uniform vec2 u_texture1InvSize;\n").unwrap();
        }
        if references.test(CombinerReference::SHADE) {
            write!(glsl, "in vec4 v_shade;\n").unwrap();
        }
        if references.test(CombinerReference::TEXEL_0)
            || references.test(CombinerReference::TEXEL_1)
        {
            write!(glsl, "in vec2 v_texCoord;\n").unwrap();
        }

        write!(
            glsl,
            r#"layout(location = 0) out vec4 fragColor;
void main() {{
"#,
        )
        .unwrap();

        if references.test(CombinerReference::PRIMITIVE) {
            let prim = self.primitive_color.unwrap_or([0; 4]);
            write!(
                glsl,
                "vec4 prim = vec4({}.0, {}.0, {}.0, {}.0) / 255.0;\n",
                prim[0], prim[1], prim[2], prim[3],
            )
            .unwrap();
        }
        if references.test(CombinerReference::ENVIRONMENT) {
            let env = self.env_color.unwrap_or([0; 4]);
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
            let prim_lod_frac = self.prim_lod_frac.unwrap_or(0);
            write!(glsl, "float primLodFrac = {}.0 / 255.0;\n", prim_lod_frac,).unwrap();
        }
        if references.test(CombinerReference::TEXEL_0) {
            self.write_glsl_for_texture(&mut glsl, '0');
        }
        if references.test(CombinerReference::TEXEL_1) {
            if self.texture_1.is_some() {
                write!(
                    glsl,
                    "vec4 texel1 = texture(u_texture1, v_texCoord * u_texture1InvSize);\n"
                )
                .unwrap();
            } else {
                write!(glsl, "vec4 texel1 = vec4(1.0, 0.0, 1.0, 1.0);\n").unwrap();
            }
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
}}
"#,
            color_ctx.get_with_ctx(color_1).unwrap(),
            alpha_ctx.get_with_ctx(alpha_1).unwrap(),
        )
        .unwrap();

        glsl
    }

    fn write_glsl_for_texture(&self, glsl: &mut String, index: char) {
        if let Some(ref texture) = self.texture_0 {
            // Align the incoming texture coordinate, in texture image coordinate space, to the
            // min-corner of tile space.
            write!(
                glsl,
                "vec2 aligned{index} = v_texCoord - vec2({min_s}.0, {min_t}.0) / 4.0;\n",
                index = index,
                min_s = texture.params.s.range.start.0,
                min_t = texture.params.t.range.start.0,
            )
            .unwrap();

            // TODO: Check and deal with any discrepancy between RDP and GL texture coordinate
            // alignment. RDP has both min-corner and texel center alignment, depending on whether
            // filtering is enabled. GL has... one of those all the time?

            // Apply clamping if enabled. Note that clamping is always enabled when not masking,
            // otherwise clamping is controlled by its flag.
            //
            // TODO: Skip this entirely if clamping is disabled on both axes?
            write!(glsl, "vec2 clamped{index} = vec2(", index = index).unwrap();
            let write_clamp_component = |glsl: &mut String, component, params: &TexCoordParams| {
                if params.mask == 0 || params.clamp {
                    // Clamping. Clamp the aligned coordinate
                    //
                    // TODO: Should this clamp to a lower bound of zero if not mirroring?
                    write!(
                        glsl,
                        "clamp(aligned{index}.{component}, -{span}.0 / 4.0, {span}.0 / 4.0)",
                        index = index,
                        component = component,
                        span = params.range.end.0 - params.range.start.0,
                    )
                    .unwrap();
                } else {
                    // Not clamping. Just pass through the aligned coordinate.
                    write!(
                        glsl,
                        "aligned{index}.{component}",
                        index = index,
                        component = component,
                    )
                    .unwrap();
                }
            };
            write_clamp_component(glsl, 's', &texture.params.s);
            write!(glsl, ", ").unwrap();
            write_clamp_component(glsl, 't', &texture.params.t);
            write!(glsl, ");\n").unwrap();

            // Skip masking for now because as written below it interferes with mirroring and with
            // the screen-space derivatives GL uses for texture filtering. Consider implementing the
            // actual RDP texture filter manually.
            write!(
                glsl,
                "vec2 masked{index} = clamped{index};\n",
                index = index
            )
            .unwrap();

            // // Apply masking.
            // //
            // // TODO: Skip this entirely if masking is disabled on both axes?
            // write!(glsl, "vec2 masked{index} = vec2(", index = index).unwrap();
            // let write_mask_component = |glsl: &mut String, component, params: &TexCoordParams| {
            //     if params.mask > 0 {
            //         write!(
            //             glsl,
            //             "fract(clamped{index}.{component} / {size}.0) * {size}.0",
            //             index = index,
            //             component = component,
            //             size = 1 << params.mask,
            //         )
            //         .unwrap();
            //     } else {
            //         write!(
            //             glsl,
            //             "clamped{index}.{component}",
            //             index = index,
            //             component = component,
            //         )
            //         .unwrap();
            //     }
            // };
            // write_mask_component(glsl, 's', &texture.params.s);
            // write!(glsl, ", ").unwrap();
            // write_mask_component(glsl, 't', &texture.params.t);
            // write!(glsl, ");\n").unwrap();

            write!(
                glsl,
                "vec4 texel{index} = texture(u_texture{index}, masked{index} * u_texture{index}InvSize);\n",
                index = index,
            )
            .unwrap();
        } else {
            write!(
                glsl,
                "vec4 texel{index} = vec4(1.0, 0.0, 1.0, 1.0);\n",
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
