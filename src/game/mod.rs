//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
mod camera;
mod map;
mod movement;
mod player;
mod shaders;

pub use camera::CameraObserver;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        movement::plugin,
        shaders::plugin,
        player::plugin,
        camera::plugin,
        map::plugin,
    ));
}

mod prelude {
    pub use super::camera::CameraTarget;
    pub use super::movement::{Movement, MovementController};
    pub use crate::screen::Screen;
}
