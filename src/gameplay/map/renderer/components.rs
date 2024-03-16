use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderGroup {
    Gameplay3D,
    PreviewMap2D,
}

#[derive(Component)]
pub struct PlayerRender;

#[derive(Component)]
pub struct RenderMap(pub HashMap<u32, Entity>);

#[derive(Component)]
pub struct CameraOffset(pub f32, pub f32, pub f32);

#[derive(Component)]
pub struct SourceCameraFollow;
