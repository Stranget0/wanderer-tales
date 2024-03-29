use bevy::{
    ecs::{entity::Entity, event::Event},
    math::Vec2,
};

use crate::gameplay::map::{
    data_source_layer::components::HexPositionFractional, utils::hex_vector::FractionalHexVector,
};

use super::components::Sight;

#[derive(Event)]
pub struct WSADEvent(pub Vec2);

#[derive(Event)]
pub struct CharacterMovedEvent {
    pub source_entity: Entity,
    pub pos: HexPositionFractional,
    pub delta_pos: FractionalHexVector,
    pub sight: Option<Sight>,
    pub is_player_controllable: bool,
}
