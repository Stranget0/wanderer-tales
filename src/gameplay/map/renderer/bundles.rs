use bevy::prelude::*;

use crate::gameplay::theme::constants::COLORS;

use super::components::CameraOffset;

const CLEAR_COLOR: ClearColorConfig = ClearColorConfig::Custom(COLORS.gray.l900);

#[derive(Bundle)]
pub struct Game3DCameraBundle(Camera3dBundle, CameraOffset);

#[derive(Bundle)]
pub struct Game2DCameraBundle(Camera2dBundle);

impl Default for Game3DCameraBundle {
    fn default() -> Self {
        Self(
            Camera3dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: CLEAR_COLOR,
                    is_active: false,
                    ..default()
                },
                ..default()
            },
            CameraOffset(0.0, 5.0, -8.0),
        )
    }
}

impl Default for Game2DCameraBundle {
    fn default() -> Self {
        Self(Camera2dBundle {
            camera: Camera {
                order: 2,
                clear_color: CLEAR_COLOR,
                is_active: false,
                ..default()
            },
            ..default()
        })
    }
}
