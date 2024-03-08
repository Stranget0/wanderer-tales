use bevy::prelude::*;

#[derive(Component)]
pub struct WSADSteerable;

#[derive(Component)]
pub struct MapSpeed(pub f32);

#[derive(Component)]
pub struct Sight(pub u16);
