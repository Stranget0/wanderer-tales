use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum RendererState {
    None,
    TwoDimension,
    ThreeDimension,
}

pub fn switch_visibility_on<T: Component>(commands: Commands, query: Query<&Visibility, With<T>>) {
    for mut visibility in query.iter() {
        visibility = &Visibility::Visible;
    }
}

pub fn switch_visibility_off<T: Component>(commands: Commands, query: Query<&Visibility, With<T>>) {
    for mut visibility in query.iter() {
        visibility = &Visibility::Hidden;
    }
}
