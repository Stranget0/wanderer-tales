use bevy::prelude::*;

use super::components::MaterialKey;

#[derive(Event)]
pub struct RenderCharacterEvent {
    pub entity: Entity,
    pub material_key: MaterialKey,
}
