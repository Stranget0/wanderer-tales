use bevy::prelude::*;

use super::components::CameraOffset;

#[derive(Bundle)]
pub struct Game3DCameraBundle(Camera3dBundle, CameraOffset);

#[derive(Bundle, Default)]
pub struct Game2DCameraBundle(Camera2dBundle);

impl Default for Game3DCameraBundle {
    fn default() -> Self {
        Self(Camera3dBundle::default(), CameraOffset(0.0, 5.0, -8.0))
    }
}
