use num_traits::identities::{One, Zero};

/// A scaled unsigned 8-bit number, where 0 -> 0.0 and 255 -> 1.0.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Su8(pub u8);
impl std::fmt::Display for Su8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_f32())
    }
}
impl std::fmt::Debug for Su8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Su8({})", self.as_f32())
    }
}
impl Su8 {
    pub fn as_f32(self) -> f32 {
        self.0 as f32 / 255.0
    }
}
impl Zero for Su8 {
    fn zero() -> Su8 {
        Su8(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl One for Su8 {
    fn one() -> Su8 {
        Su8(255)
    }
    fn is_one(&self) -> bool {
        self.0 == 255
    }
}
impl std::ops::Add<Su8> for Su8 {
    type Output = Su8;
    fn add(self, rhs: Su8) -> Su8 {
        Su8(self.0.checked_add(rhs.0).unwrap())
    }
}
impl std::ops::AddAssign<Su8> for Su8 {
    fn add_assign(&mut self, rhs: Su8) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}
impl std::ops::Mul<Su8> for Su8 {
    type Output = Su8;
    fn mul(self, rhs: Su8) -> Su8 {
        let long = (self.0 as u16).checked_mul(rhs.0 as u16).unwrap();
        let units = long / 255;
        let remainder = long - 255 * units;
        Su8((units + remainder / 128) as u8)
    }
}
impl std::ops::MulAssign<Su8> for Su8 {
    fn mul_assign(&mut self, rhs: Su8) {
        *self = *self * rhs;
    }
}
impl std::ops::Neg for Su8 {
    type Output = Su8;
    fn neg(self) -> Su8 {
        match self.0 {
            0 => Su8(0),
            _ => panic!("result would be out of range: !{:?}", self),
        }
    }
}
#[cfg(test)]
mod su8_tests {
    use super::Su8;

    #[test]
    fn add() {
        assert_eq!(Su8(3) + Su8(5), Su8(8));
    }

    #[test]
    #[should_panic]
    fn add_panics_on_overflow() {
        let _ = Su8(128) + Su8(128);
    }

    #[test]
    fn mul() {
        assert_eq!(Su8(0) * Su8(0), Su8(0));
        assert_eq!(Su8(0) * Su8(255), Su8(0));
        assert_eq!(Su8(255) * Su8(0), Su8(0));
        assert_eq!(Su8(255) * Su8(255), Su8(255));
        assert_eq!(Su8(85) * Su8(170), Su8(57));
    }
}
