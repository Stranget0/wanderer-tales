use std::ops::{Add, Mul, Sub};

use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Vec3Lerpable(pub Vec3);

impl bevy_easings::Lerp for Vec3Lerpable {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        let x = interpolation::lerp(&self.0.x, &other.0.x, scalar);
        let y = interpolation::lerp(&self.0.y, &other.0.y, scalar);
        let z = interpolation::lerp(&self.0.z, &other.0.z, scalar);

        Self(Vec3::new(x, y, z))
    }
}

impl Add for Vec3Lerpable {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3Lerpable(self.0 + rhs.0)
    }
}

impl Sub for Vec3Lerpable {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3Lerpable(self.0 - rhs.0)
    }
}

impl Mul<f32> for Vec3Lerpable {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3Lerpable(self.0 * rhs)
    }
}
