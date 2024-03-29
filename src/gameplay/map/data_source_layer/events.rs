use bevy::ecs::{entity::Entity, event::Event};

use crate::gameplay::map::{
    renderer::components::RenderGroup, utils::hex_map_item::HexMapTileBundle,
};

#[derive(Event)]
pub struct MapAddEvent {
    pub source_items: Vec<(Entity, HexMapTileBundle)>,
    pub render_groups: Vec<RenderGroup>,
}

#[derive(Event)]
pub struct MapSubEvent {
    pub source_items: Vec<Entity>,
    pub render_groups: Vec<RenderGroup>,
}
