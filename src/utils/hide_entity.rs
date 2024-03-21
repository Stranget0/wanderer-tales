use bevy::prelude::*;

pub fn hide_entity<E: Component>(mut commands: Commands, query: Query<Entity, With<E>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
