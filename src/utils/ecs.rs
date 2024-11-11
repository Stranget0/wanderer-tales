use bevy::prelude::*;

pub fn toggle_visibility<T: Component>(mut query: Query<&mut Visibility, With<T>>) {
    for mut visibility in query.iter_mut() {
        let new_visibility = match visibility.as_ref() {
            Visibility::Visible | Visibility::Inherited => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,
        };

        *visibility = new_visibility;
    }
}

pub fn despawn_entities<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    info!("Despawning {} entities", query.iter().count());
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// A condition that returns true when components of type T are added to any entity
pub fn components_added<T: Component>(query: Query<(), Added<T>>) -> bool {
    !query.is_empty()
}

/// A condition that returns true when components of type T are removed from any entity
pub fn components_removed<T: Component>(query: RemovedComponents<T>) -> bool {
    !query.is_empty()
}

/// A condition that returns true when components of type T are changed in any entity
pub fn components_changed<T: Component>(query: Query<(), Changed<T>>) -> bool {
    !query.is_empty()
}
