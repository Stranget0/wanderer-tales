use bevy::prelude::*;

use crate::gameplay::map::renderer::state::RendererState;

pub fn debug_switch_renderer(
    state: Res<State<RendererState>>,
    mut next_state: ResMut<NextState<RendererState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        let new_state = match state.get() {
            RendererState::None => RendererState::ThreeDimension,
            RendererState::TwoDimension => RendererState::ThreeDimension,
            RendererState::ThreeDimension => RendererState::TwoDimension,
        };
        next_state.set(new_state);
    }
}
