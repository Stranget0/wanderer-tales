use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn spawn_default_with_parent<T: Bundle + Default, F: QueryFilter>(
    mut commands: Commands,
    parent_query: Query<Entity, F>,
) {
    let entity = commands.spawn(T::default()).id();
    match parent_query.get_single() {
        Ok(parent_entity) => {
            commands.entity(parent_entity).add_child(entity);
        }
        Err(err) => {
            error!("{}", err);
        }
    };
}

pub fn despawn_with_parent<T: QueryFilter>(mut commands: Commands, entity_query: Query<Entity, T>) {
    for entity in entity_query.iter() {
        commands.entity(entity).remove_parent();
        commands.entity(entity).despawn_recursive();
    }
}

pub fn hide_entity<E: Component>(mut commands: Commands, query: Query<Entity, With<E>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
