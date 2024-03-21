use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn_with_parent<T: QueryFilter>(mut commands: Commands, entity_query: Query<Entity, T>) {
    for entity in entity_query.iter() {
        commands.entity(entity).remove_parent();
        commands.entity(entity).despawn_recursive();
    }
}
