use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct CameraOffset(pub f32, pub f32, pub f32);

impl From<CameraOffset> for Vec3 {
    fn from(val: CameraOffset) -> Self {
        Vec3::new(val.0, val.1, val.2)
    }
}

impl From<&CameraOffset> for Vec3 {
    fn from(val: &CameraOffset) -> Self {
        Vec3::new(val.0, val.1, val.2)
    }
}

#[derive(Component, Debug, Default)]
pub struct CameraRotation(pub f32, pub f32, pub f32);

#[derive(Component)]
pub struct SourceCameraFollow;
