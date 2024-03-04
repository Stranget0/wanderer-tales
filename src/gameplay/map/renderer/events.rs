use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Event)]
pub struct RenderPointEvent {
    pub parent: Entity,
    pub color: Color,
    pub size: f32,
}

#[derive(Event)]
pub struct RenderHexEvent {
    pub entity: Entity,
    pub bundle: MaterialMesh2dBundle<ColorMaterial>,
}
