use bevy::prelude::*;

use super::components::MaterialKey;

#[derive(Event)]
pub struct RenderCharacter {
    pub entity: Entity,
    pub material_key: MaterialKey,
}
