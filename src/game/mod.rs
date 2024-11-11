//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
mod character_controller;
mod lights;
pub mod map;
mod player_controller;
mod shaders;

use character_controller::*;
pub use player_controller::{
    controls_locked, CameraOrbit, CameraOrbitTarget, ControlLock, ControlLocks,
};

pub mod devtools {
    pub use super::map::map_devtools::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        lights::plugin,
        audio::plugin,
        assets::plugin,
        shaders::plugin,
        character_controller::plugin,
        player_controller::plugin,
        map::plugin,
    ));
}
