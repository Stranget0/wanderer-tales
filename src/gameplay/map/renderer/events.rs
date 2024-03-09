use bevy::prelude::*;

use super::components::MaterialKey;

#[derive(Event)]
pub struct RenderCharacterEvent {
    pub parent: Entity,
    pub material_key: MaterialKey,
}
