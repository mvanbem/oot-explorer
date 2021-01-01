use num_traits::{One, Zero};
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum GlslFloatConstant {
    Zero,
    One,
}
impl Display for GlslFloatConstant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GlslFloatConstant::Zero => write!(f, "0.0"),
            GlslFloatConstant::One => write!(f, "1.0"),
        }
    }
}
impl Zero for GlslFloatConstant {
    fn zero() -> GlslFloatConstant {
        GlslFloatConstant::Zero
    }
    fn is_zero(&self) -> bool {
        *self == GlslFloatConstant::Zero
    }
}
impl One for GlslFloatConstant {
    fn one() -> GlslFloatConstant {
        GlslFloatConstant::One
    }
    fn is_one(&self) -> bool {
        *self == GlslFloatConstant::One
    }
}
impl Add for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn add(self, rhs: GlslFloatConstant) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match (self, rhs) {
            (Zero, Zero) => Zero,
            (Zero, One) | (One, Zero) => One,
            (One, One) => panic!("overflow"),
        }
    }
}
impl AddAssign for GlslFloatConstant {
    fn add_assign(&mut self, rhs: GlslFloatConstant) {
        *self = *self + rhs;
    }
}
impl Mul for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn mul(self, rhs: GlslFloatConstant) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match (self, rhs) {
            (One, One) => One,
            _ => Zero,
        }
    }
}
impl MulAssign for GlslFloatConstant {
    fn mul_assign(&mut self, rhs: GlslFloatConstant) {
        *self = *self * rhs;
    }
}
impl Neg for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn neg(self) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match self {
            Zero => Zero,
            One => panic!("overflow"),
        }
    }
}
