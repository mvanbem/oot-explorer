use num_traits::{One, Zero};
use oot_explorer_expr as expr;
use oot_explorer_game_data::gbi::{AlphaCombine, AlphaInput, ColorCombine, ColorInput};

use crate::glsl_float_constant::GlslFloatConstant;
use crate::glsl_vec3_constant::GlslVec3Constant;
use crate::rcp::Cycle;

pub trait ToExpr<T: expr::ValueType> {
    fn to_expr(&self, ctx: &mut expr::Context<T>, cycle: Cycle) -> expr::Key;
}

impl ToExpr<GlslVec3Constant> for ColorCombine {
    fn to_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>, cycle: Cycle) -> expr::Key {
        let a = self.a.to_expr(ctx, cycle);
        let b = self.b.to_expr(ctx, cycle);
        let neg_b = ctx.neg(b);
        let sum = ctx.add(vec![a, neg_b]);
        let c = self.c.to_expr(ctx, cycle);
        let product = ctx.mul(vec![sum, c]);
        let d = self.d.to_expr(ctx, cycle);
        ctx.add(vec![product, d])
    }
}

impl ToExpr<GlslFloatConstant> for AlphaCombine {
    fn to_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>, cycle: Cycle) -> expr::Key {
        let a = self.a.to_expr(ctx, cycle);
        let b = self.b.to_expr(ctx, cycle);
        let neg_b = ctx.neg(b);
        let sum = ctx.add(vec![a, neg_b]);
        let c = self.c.to_expr(ctx, cycle);
        let product = ctx.mul(vec![sum, c]);
        let d = self.d.to_expr(ctx, cycle);
        ctx.add(vec![product, d])
    }
}

impl ToExpr<GlslVec3Constant> for ColorInput {
    fn to_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>, cycle: Cycle) -> expr::Key {
        match self {
            ColorInput::Combined => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("combined.rgb".to_string()),
            },
            ColorInput::Texel0 => ctx.symbol("texel0.rgb".to_string()),
            ColorInput::Texel1 => ctx.symbol("texel1.rgb".to_string()),
            ColorInput::Primitive => ctx.symbol("prim.rgb".to_string()),
            ColorInput::Shade => ctx.symbol("v_shade.rgb".to_string()),
            ColorInput::Environment => ctx.symbol("env.rgb".to_string()),
            ColorInput::One => ctx.literal(One::one()),
            ColorInput::Noise => ctx.symbol("noise.rgb".to_string()),
            ColorInput::Zero => ctx.literal(Zero::zero()),
            ColorInput::Center => ctx.symbol("u_center".to_string()),
            ColorInput::K4 => ctx.symbol("vec3(u_k4)".to_string()),
            ColorInput::Scale => ctx.symbol("u_scale".to_string()),
            ColorInput::CombinedAlpha => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("combined.aaa".to_string()),
            },
            ColorInput::Texel0Alpha => ctx.symbol("texel0.aaa".to_string()),
            ColorInput::Texel1Alpha => ctx.symbol("texel1.aaa".to_string()),
            ColorInput::PrimitiveAlpha => ctx.symbol("prim.aaa".to_string()),
            ColorInput::ShadeAlpha => ctx.symbol("shade.aaa".to_string()),
            ColorInput::EnvAlpha => ctx.symbol("env.aaa".to_string()),
            ColorInput::LodFraction => ctx.symbol("vec3(lodFraction)".to_string()),
            ColorInput::PrimLodFrac => ctx.symbol("vec3(primLodFrac)".to_string()),
            ColorInput::K5 => ctx.symbol("vec3(u_k5)".to_string()),
        }
    }
}

impl ToExpr<GlslFloatConstant> for AlphaInput {
    fn to_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>, cycle: Cycle) -> expr::Key {
        match self {
            AlphaInput::Combined => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("combined.a".to_string()),
            },
            AlphaInput::Texel0 => ctx.symbol("texel0.a".to_string()),
            AlphaInput::Texel1 => ctx.symbol("texel1.a".to_string()),
            AlphaInput::Primitive => ctx.symbol("prim.a".to_string()),
            AlphaInput::Shade => ctx.symbol("v_shade.a".to_string()),
            AlphaInput::Environment => ctx.symbol("env.a".to_string()),
            AlphaInput::One => ctx.literal(One::one()),
            AlphaInput::Zero => ctx.literal(Zero::zero()),
            AlphaInput::LodFraction => ctx.symbol("lodFraction".to_string()),
            AlphaInput::PrimLodFrac => ctx.symbol("primLodFrac".to_string()),
        }
    }
}

#[cfg(test)]
mod to_expr_tests {
    use super::*;

    #[test]
    fn test() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "(combined.rgb - texel0.rgb) * texel1.rgb + prim.rgb"
        );
    }

    #[test]
    fn a_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Zero,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "-texel0.rgb * texel1.rgb + prim.rgb"
        );
    }

    #[test]
    fn b_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Zero,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "combined.rgb * texel1.rgb + prim.rgb"
        );
    }

    #[test]
    fn c_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Zero,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(format!("{:?}", ctx.get_with_ctx(expr).unwrap()), "prim.rgb");
    }

    #[test]
    fn c_one() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::One,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "combined.rgb - texel0.rgb + prim.rgb"
        );
    }

    #[test]
    fn d_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Zero,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "(combined.rgb - texel0.rgb) * texel1.rgb"
        );
    }
}
