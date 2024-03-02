use bevy::{ecs::event::Event, math::Vec2};

#[derive(Event)]
pub struct WSADEvent(pub Vec2);
