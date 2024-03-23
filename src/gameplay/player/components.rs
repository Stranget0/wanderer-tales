use bevy::prelude::*;

use crate::gameplay::map::utils::hex_vector::{FractionalHexVector, HexVector};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct PlayerRoot;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct PlayerControllable;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct WSADSteerable;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct MapSpeed(pub f32);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Sight(pub u16);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct HexPositionFractional(pub FractionalHexVector);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct HexPositionFractionalDelta(pub FractionalHexVector);

impl Default for HexPositionFractionalDelta {
    fn default() -> Self {
        Self(FractionalHexVector(0.0, 0.0, 0.0))
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct HexPosition(pub HexVector);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct HexPositionDelta(pub HexVector);

impl Default for HexPositionDelta {
    fn default() -> Self {
        Self(HexVector(0, 0, 0))
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Rotation(pub Vec3);

#[derive(Component)]
pub struct Character;
