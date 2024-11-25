use crate::prelude::*;

mod actions;
mod camera;
mod player;

pub use actions::{controls_locked, ControlLock, ControlLocks};
use bevy::window::PrimaryWindow;
pub use camera::{CameraOrbit, CameraOrbitTarget};

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins((camera::plugin, actions::plugin, player::plugin))
        .add_systems(
            Update,
            derive_mouse_visibility
                .run_if(resource_changed::<ControlLocks>.or_else(state_changed::<GameState>))
                .in_set(GameSet::Update),
        );
}

fn derive_mouse_visibility(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    control_locks: Res<ControlLocks>,
    game_state: Res<State<GameState>>,
) {
    for mut window in primary_window.iter_mut() {
        window.cursor.visible =
            !control_locks.0.is_empty() || game_state.get() != &GameState::Playing;
    }
}
