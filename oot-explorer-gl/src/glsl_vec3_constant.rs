use num_traits::{One, Zero};
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum GlslVec3Constant {
    Zero,
    One,
}
impl Display for GlslVec3Constant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GlslVec3Constant::Zero => write!(f, "vec3(0.0, 0.0, 0.0)"),
            GlslVec3Constant::One => write!(f, "vec3(1.0, 1.0, 1.0)"),
        }
    }
}
impl Zero for GlslVec3Constant {
    fn zero() -> GlslVec3Constant {
        GlslVec3Constant::Zero
    }
    fn is_zero(&self) -> bool {
        *self == GlslVec3Constant::Zero
    }
}
impl One for GlslVec3Constant {
    fn one() -> GlslVec3Constant {
        GlslVec3Constant::One
    }
    fn is_one(&self) -> bool {
        *self == GlslVec3Constant::One
    }
}
impl Add for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn add(self, rhs: GlslVec3Constant) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match (self, rhs) {
            (Zero, Zero) => Zero,
            (Zero, One) | (One, Zero) => One,
            (One, One) => panic!("overflow"),
        }
    }
}
impl AddAssign for GlslVec3Constant {
    fn add_assign(&mut self, rhs: GlslVec3Constant) {
        *self = *self + rhs;
    }
}
impl Mul for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn mul(self, rhs: GlslVec3Constant) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match (self, rhs) {
            (One, One) => One,
            _ => Zero,
        }
    }
}
impl MulAssign for GlslVec3Constant {
    fn mul_assign(&mut self, rhs: GlslVec3Constant) {
        *self = *self * rhs;
    }
}
impl Neg for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn neg(self) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match self {
            Zero => Zero,
            One => panic!("overflow"),
        }
    }
}
