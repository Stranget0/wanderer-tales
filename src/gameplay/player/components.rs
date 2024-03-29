use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct PlayerRoot;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct PlayerControllable;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct WSADSteerable;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct MouseRotatable(pub f32);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct MapSpeed(pub f32);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Sight(pub u16);

#[derive(Component)]
pub struct Character;
