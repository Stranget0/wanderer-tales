use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn remove_component<F: QueryFilter, C: Component>(
    mut commands: Commands,
    query: Query<Entity, F>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<C>();
    }
}
