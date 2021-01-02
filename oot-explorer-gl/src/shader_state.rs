use oot_explorer_core::gbi::{TextureDepth, TextureFormat};
use oot_explorer_expr as expr;

use crate::glsl_float_constant::GlslFloatConstant;
use crate::glsl_vec3_constant::GlslVec3Constant;
use crate::rcp::{CombinerState, Cycle, TmemSource};
use crate::to_expr::ToExpr;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ShaderState {
    pub two_cycle_mode: bool,
    pub combiner: CombinerState,
    pub texture_a: Option<TextureDescriptor>,
    pub texture_b: Option<TextureDescriptor>,
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

        format!(
            r#"#version 300 es

precision highp float;
precision highp int;

uniform vec4 u_env;
/*
uniform vec3 u_center;
uniform vec3 u_scale;
uniform float u_k4;
uniform float u_k5;
*/

in vec4 v_color;
in vec4 v_shade;

layout(location = 0) out vec4 fragColor;

void main() {{
  // TODO: implement texturing
  vec4 texel0 = vec4(1.0, 0.0, 1.0, 0.5);
  vec4 texel1 = vec4(1.0, 0.0, 1.0, 0.5);
  // TODO: implement noise?
  // TODO: implement LOD
  float lod_fraction = 0.5;
  float prim_lod_frac = 0.5;

  vec4 cycle1 = vec4({:?}, {:?});
  fragColor = vec4({:?}, {:?});
}}
"#,
            color_ctx.get_with_ctx(color_0).unwrap(),
            alpha_ctx.get_with_ctx(alpha_0).unwrap(),
            color_ctx.get_with_ctx(color_1).unwrap(),
            alpha_ctx.get_with_ctx(alpha_1).unwrap(),
        )
    }
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
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PaletteSource {
    None,
    Rgba(TmemSource),
    Ia(TmemSource),
}
