use bevy::ecs::{entity::Entity, event::Event};

use crate::features::map::utils::hex_vector::HexVector;

use super::utils::hex_map_item::HexMapItemBundle;

#[derive(Event)]
pub struct MoveMapOriginEvent(pub HexVector);

#[derive(Event)]
pub struct MapAddEvent(pub Vec<Entity>);
