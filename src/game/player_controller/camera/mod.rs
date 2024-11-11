pub(super) mod control;

pub use control::{CameraOrbit, CameraOrbitTarget};

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins(control::plugin);
}
