use bevy::ecs::schedule::States;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum RendererState {
    None,
    #[default]
    TwoDimension,
    ThreeDimension,
}
