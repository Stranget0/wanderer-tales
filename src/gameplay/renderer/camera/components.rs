use bevy::prelude::*;

use crate::utils::{RangeBetween, Vec3Lerpable};

#[derive(Component)]
pub struct SourceCameraFollow;

#[derive(Component)]
pub struct CameraSlide(pub RangeBetween<Vec3Lerpable>);

#[derive(Component)]
pub struct CameraRotationRange(pub RangeBetween<f32>, pub RangeBetween<f32>);

#[derive(Component, Default)]
pub struct CameraOffset(pub Transform);
