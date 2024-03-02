use bevy::prelude::*;

#[derive(Event)]
pub struct RenderPointEvent {
    pub parent: Entity,
    pub color: Color,
    pub size: f32,
}
