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
