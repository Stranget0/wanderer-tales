pub(super) mod control;

use crate::prelude::*;
pub use control::*;

pub fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins(control::plugin);
}

pub fn has_camera_focus_moved(
    camera_query: Query<&Transform, (Changed<Transform>, With<CameraFocus>)>,
) -> bool {
    !camera_query.is_empty()
}
