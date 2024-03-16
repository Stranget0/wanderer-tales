use bevy::prelude::*;

use crate::gameplay::player::components::HexPositionFractional;

use super::utils::MaterialKey;

#[derive(Event)]
pub struct RenderCharacterEvent {
    pub source_entity: Entity,
    pub material_key: MaterialKey,
    pub position: HexPositionFractional,
}
