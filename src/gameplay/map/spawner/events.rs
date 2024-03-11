use bevy::ecs::{entity::Entity, event::Event};

use crate::gameplay::map::renderer::components::RenderGroup;

pub trait MapChangeEvent {
    fn get_items(&self) -> &Vec<Entity>;
    fn get_render_groups(&self) -> &Vec<RenderGroup>;
}

#[derive(Event)]
pub struct MapAddEvent {
    pub source_items: Vec<Entity>,
    pub render_groups: Vec<RenderGroup>,
}

impl MapChangeEvent for MapAddEvent {
    fn get_items(&self) -> &Vec<Entity> {
        &self.source_items
    }

    fn get_render_groups(&self) -> &Vec<RenderGroup> {
        &self.render_groups
    }
}

#[derive(Event)]
pub struct MapSubEvent {
    pub source_items: Vec<Entity>,
    pub render_groups: Vec<RenderGroup>,
}

impl MapChangeEvent for MapSubEvent {
    fn get_items(&self) -> &Vec<Entity> {
        &self.source_items
    }

    fn get_render_groups(&self) -> &Vec<RenderGroup> {
        &self.render_groups
    }
}
