pub use bevy::math::*;
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueDt2 {
    pub value: f32,
    pub derivative: Dt2,
}
impl ValueDt2 {
    pub fn new(value: f32, derivative: Vec2) -> Self {
        Self {
            value,
            derivative: Dt2(derivative),
        }
    }
    pub fn get_normal(&self) -> Vec3 {
        self.derivative.get_normal()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dt2(pub Vec2);
impl Dt2 {
    pub fn get_normal(&self) -> Vec3 {
        vec3(-self.0.x, 1.0, -self.0.y).normalize()
    }
    pub fn length(&self) -> f32 {
        self.0.length()
    }
}
impl Add<Dt2> for Dt2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Dt2 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (*self + rhs).0;
    }
}
impl Sub<Dt2> for Dt2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}
impl Mul<Dt2> for Dt2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}
impl Div<Dt2> for Dt2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

// impl Add<f32> for ValueDt2 {
//     type Output = Self;
//     fn add(self, rhs: f32) -> Self {
//         Self::new(self.value + rhs, self.derivative)
//     }
// }
//
// impl Sub<f32> for ValueDt2 {
//     type Output = Self;
//     fn sub(self, rhs: f32) -> Self {
//         Self::new(self.value - rhs, self.derivative)
//     }
// }
//
// impl Mul<f32> for ValueDt2 {
//     type Output = Self;
//     fn mul(self, rhs: f32) -> Self {
//         Self::new(self.value * rhs, self.derivative * rhs)
//     }
// }
// impl Div<f32> for ValueDt2 {
//     type Output = Self;
//     fn div(self, rhs: f32) -> Self {
//         Self::new(self.value / rhs, self.derivative / rhs)
//     }
// }
//
// impl Add<ValueDt2> for ValueDt2 {
//     type Output = Self;
//     fn add(self, rhs: Self) -> Self {
//         Self::new(self.value + rhs.value, self.derivative + rhs.derivative)
//     }
// }
//
// impl Sub<ValueDt2> for ValueDt2 {
//     type Output = Self;
//     fn sub(self, rhs: Self) -> Self {
//         Self::new(self.value - rhs.value, self.derivative - rhs.derivative)
//     }
// }
//
// impl Mul<ValueDt2> for ValueDt2 {
//     type Output = Self;
//     fn mul(self, rhs: Self) -> Self {
//         let derivative = self.derivative * rhs.value + rhs.derivative * self.value;
//         let value = self.value * rhs.value;
//         Self::new(value, derivative)
//     }
// }

pub struct ValueDt3 {
    pub value: f32,
    pub derivative: Vec3,
}
impl ValueDt3 {
    pub fn new(value: f32, derivative: Vec3) -> Self {
        Self { value, derivative }
    }
}

// Wrapping pcg function
fn pcg(n: u32) -> u32 {
    let mut h = n.wrapping_mul(747796405).wrapping_add(2891336453);
    h = (h >> ((h >> 28).wrapping_add(4)) ^ h).wrapping_mul(277803737);
    (h >> 22) ^ h
}

fn rand11(f: f32) -> f32 {
    pcg(f.to_bits()) as f32 / 0xffffffff_u32 as f32
}

fn rand21(p: Vec2) -> f32 {
    let n = p.x * 3.0 + p.y * 113.0;
    rand11(n)
}

// Value noise and gradient noise
pub fn value_noise_2d(p: Vec2) -> ValueDt2 {
    let i = p.floor();
    let f = p.fract_gl();

    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);

    let va = rand21(i + vec2(0.0, 0.0));
    let vb = rand21(i + vec2(1.0, 0.0));
    let vc = rand21(i + vec2(0.0, 1.0));
    let vd = rand21(i + vec2(1.0, 1.0));

    ValueDt2::new(
        va + (vb - va) * u.x + (vc - va) * u.y + (va - vb - vc + vd) * u.x * u.y,
        du * (u.yx() * (va - vb - vc + vd) + vec2(vb, vc) - va),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_noise_2d() {
        for x in -10000..10000 {
            for y in -10000..10000 {
                let v = value_noise_2d(vec2(x as f32 / 100.0, y as f32 / 100.0));
                assert!(v.value <= 1.0);
                assert!(v.value >= 0.0);
            }
        }
    }
}
