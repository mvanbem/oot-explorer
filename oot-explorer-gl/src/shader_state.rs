use oot_explorer_core::gbi::{TextureDepth, TextureFormat};
use oot_explorer_expr as expr;
use std::fmt::Write;

use crate::glsl_float_constant::GlslFloatConstant;
use crate::glsl_vec3_constant::GlslVec3Constant;
use crate::rcp::{CombinerState, Cycle, TmemSource};
use crate::to_expr::ToExpr;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ShaderState {
    pub two_cycle_mode: bool,
    pub env: [u8; 4],
    pub combiner: CombinerState,
    pub texture_a: Option<TextureUsage>,
    pub texture_b: Option<TextureUsage>,
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

        let mut glsl = String::new();
        write!(
            glsl,
            r#"#version 300 es
            precision highp float;
            precision highp int;
        "#,
        )
        .unwrap();

        // TODO: Add these only when referenced.
        if self.texture_a.is_some() {
            write!(glsl, "uniform sampler2D u_texture_a;\n").unwrap();
            write!(glsl, "uniform vec2 u_texture_a_inv_size;\n").unwrap();
        }
        if self.texture_b.is_some() {
            write!(glsl, "uniform sampler2D u_texture_b;\n").unwrap();
            write!(glsl, "uniform vec2 u_texture_b_inv_size;\n").unwrap();
        }
        write!(glsl, "in vec4 v_color;\n").unwrap();
        write!(glsl, "in vec4 v_shade;\n").unwrap();
        write!(glsl, "in vec2 v_tex_coord;\n").unwrap();

        write!(
            glsl,
            r#"layout(location = 0) out vec4 fragColor;
void main() {{
"#,
        )
        .unwrap();

        // TODO: Add these only when referenced.
        write!(
            glsl,
            "vec4 env = vec4({}.0 / 255.0, {}.0 / 255.0, {}.0 / 255.0, {}.0 / 255.0);\n",
            self.env[0], self.env[1], self.env[2], self.env[3],
        )
        .unwrap();
        if self.texture_a.is_some() {
            write!(
                glsl,
                "vec4 texel0 = texture(u_texture_a, v_tex_coord * u_texture_a_inv_size);\n"
            )
            .unwrap();
        } else {
            write!(glsl, "vec4 texel0 = vec4(1.0, 0.0, 1.0, 1.0);\n").unwrap();
        }
        if self.texture_b.is_some() {
            write!(
                glsl,
                "vec4 texel1 = texture(u_texture_b, v_tex_coord * u_texture_b_inv_size);\n"
            )
            .unwrap();
        } else {
            write!(glsl, "vec4 texel1 = vec4(1.0, 0.0, 1.0, 1.0);\n").unwrap();
        }

        // TODO: What's the best way to high-level emulate mipmapping? Does OoT use mipmapping?
        write!(
            glsl,
            r#"float lod_fraction = 0.0;
  float prim_lod_frac = 0.0;
  "#
        )
        .unwrap();

        // Emit the two-cycle color combiner expressions.
        write!(
            glsl,
            r#"vec4 cycle1 = vec4({:?}, {:?});
fragColor = vec4({:?}, {:?});
}}
"#,
            color_ctx.get_with_ctx(color_0).unwrap(),
            alpha_ctx.get_with_ctx(alpha_0).unwrap(),
            color_ctx.get_with_ctx(color_1).unwrap(),
            alpha_ctx.get_with_ctx(alpha_1).unwrap(),
        )
        .unwrap();

        glsl
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureUsage {
    pub descriptor: TextureDescriptor,
    pub params_s: TexCoordParams,
    pub params_t: TexCoordParams,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TexCoordParams {
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
