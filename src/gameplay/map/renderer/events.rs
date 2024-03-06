use bevy::prelude::*;

use super::renderer_2d::resources::MaterialKey;

#[derive(Event)]
pub struct RenderCharacter {
    pub entity: Entity,
    pub material_key: MaterialKey,
}
