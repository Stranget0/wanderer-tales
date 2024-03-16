use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum RendererState {
    None,
    TwoDimension,
    ThreeDimension,
}

pub fn switch_visibility_on<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

pub fn switch_visibility_off<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
