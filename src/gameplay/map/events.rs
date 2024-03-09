use bevy::ecs::{entity::Entity, event::Event};

#[derive(Event, Debug)]
pub struct SpawnMap;

#[derive(Event, Debug)]
pub struct ClearMap(pub Entity);
