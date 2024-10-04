//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
mod camera;
mod lights;
mod map;
mod movement;
mod player;
mod shaders;

pub use camera::CameraOrbit;

pub mod devtools {
    pub use super::map::map_devtools::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        lights::plugin,
        audio::plugin,
        assets::plugin,
        shaders::plugin,
        movement::plugin,
        player::plugin,
        camera::plugin,
        map::plugin,
    ));
}

mod prelude {
    pub use crate::screen::Screen;
}
