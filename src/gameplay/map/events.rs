use bevy::ecs::{entity::Entity, event::Event};

use super::utils::hex_vector::HexVector;

#[derive(Event)]
pub struct MoveMapOriginEvent(pub HexVector);

#[derive(Event)]
pub struct MapAddEvent(pub Vec<Entity>);
