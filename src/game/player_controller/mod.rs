mod actions;
mod camera;
mod player;

pub use actions::{controls_locked, ControlLock, ControlLocks};
pub use camera::{CameraOrbit, CameraOrbitTarget};

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins((camera::plugin, actions::plugin, player::plugin));
}
