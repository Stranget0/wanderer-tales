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
