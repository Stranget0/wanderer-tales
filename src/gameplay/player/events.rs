use bevy::{
    ecs::{entity::Entity, event::Event},
    math::Vec2,
};

use super::components::{HexPositionFractional, HexPositionFractionalDelta, Sight};

#[derive(Event)]
pub struct WSADEvent(pub Vec2);

#[derive(Event)]
pub struct CharacterMovedEvent {
    pub source_entity: Entity,
    pub pos: HexPositionFractional,
    pub delta_pos: HexPositionFractionalDelta,
    pub sight: Option<Sight>,
    pub is_player_controllable: bool,
}

#[derive(Event)]
pub struct PlayerWithSightSpawnedEvent {
    pub sight: Sight,
    pub pos: HexPositionFractional,
}
