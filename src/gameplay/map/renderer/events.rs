use bevy::prelude::*;

use crate::gameplay::player::components::HexPositionFractional;

use super::{components::RenderGroup, utils::MaterialKey};

#[derive(Event)]
pub struct RenderCharacterEvent {
    pub character_entity: Entity,
    pub material_key: MaterialKey,
    pub position: HexPositionFractional,
    pub render_groups: Vec<RenderGroup>,
}
