//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
pub mod character_controller;
mod lights;
pub mod map;
pub mod physics;
mod player_controller;
mod shaders;

use character_controller::*;
pub use player_controller::{
    actions, actions::CameraAction, actions::PlayerAction, camera, controls_locked, CameraOrbit,
    CameraOrbitTarget, ControlLock, ControlLocks,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        lights::plugin,
        audio::plugin,
        assets::plugin,
        shaders::plugin,
        physics::plugin,
        character_controller::plugin,
        player_controller::plugin,
        map::plugin,
    ));
}
