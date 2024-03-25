use super::map::utils::*;
use crate::utils::{EULER_ROT, FORWARD};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct HexPositionFractional(pub FractionalHexVector);

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct HexPosition(pub HexVector);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct HexPositionDelta(pub HexVector);

impl Default for HexPositionDelta {
    fn default() -> Self {
        Self(HexVector(0, 0, 0))
    }
}

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct Rotation(pub Quat);

impl Rotation {
    pub fn from_vec(vec: Vec3) -> Self {
        Self(Quat::from_euler(EULER_ROT, vec.x, vec.y, vec.z))
    }

    pub fn get_rotated_vec3(&self, vec: &Vec2) -> Vec3 {
        let direction_3d = vec.x * Vec3::X + vec.y * FORWARD;

        self.0.mul_vec3(direction_3d)
    }

    pub fn get_rotated_vec2(&self, vec: &Vec2) -> Vec2 {
        let vec_3d = self.get_rotated_vec3(vec);

        Vec2::new(vec_3d.x, vec_3d.y)
    }
}