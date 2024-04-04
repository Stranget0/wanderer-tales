use bevy::prelude::*;

use crate::{
    gameplay::theme::constants::COLORS,
    utils::{to_3d_space, RangeBetween, Vec3Lerpable},
};

use super::components::*;

const CLEAR_COLOR: ClearColorConfig = ClearColorConfig::Custom(COLORS.gray.l900);

#[derive(Bundle)]
pub struct Game3DCameraBundle(Camera3dBundle, CameraSlide, CameraRotationRange);

#[derive(Bundle)]
pub struct Game2DCameraBundle(Camera2dBundle);

impl Default for Game3DCameraBundle {
    fn default() -> Self {
        let [x, y, z] = to_3d_space(0.0, -3.0, 3.0);
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
            CameraSlide(RangeBetween::new(
                Vec3Lerpable(Vec3::new(0.0, 1.0, 3.0)),
                Vec3Lerpable(Vec3::new(0.0, 120.0, 360.0)),
                0.3,
            )),
            CameraRotationRange(
                RangeBetween::new(-180.0, 180.0, 0.5),
                RangeBetween::new(25.0, 165.0, 0.5),
            ),
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
