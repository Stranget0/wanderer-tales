pub use bevy::math::*;
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueDt2 {
    pub derivative: Dt2,
    pub value: f32,
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

pub struct ValueDt3 {
    pub value: f32,
    pub derivative: Vec3,
}

impl ValueDt3 {
    pub fn new(value: f32, derivative: Vec3) -> Self {
        Self { value, derivative }
    }
}
