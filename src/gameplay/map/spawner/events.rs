use bevy::{
    ecs::{entity::Entity, event::Event},
    math::Vec2,
};

#[derive(Event, Debug)]
pub struct MoveSightEvent {
    pub pos: Vec2,
    pub delta_pos: Vec2,
    pub sight: u16,
    pub force_render: bool,
    pub map_display: Entity,
}

impl Default for MoveSightEvent {
    fn default() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.0),
            delta_pos: Vec2::new(0.0, 0.0),
            sight: 1,
            force_render: false,
            map_display: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Event)]
pub struct MapAddEvent(pub Vec<Entity>);
