//! Development tools for the game. This plugin is only enabled in dev builds.
pub mod debug_flags;
pub mod debug_normals;
pub mod editor_ui;
pub mod gizmos;
pub mod logs;
pub mod wireframe;

use crate::{game, prelude::*};

pub fn minimal_dev_tools_plugin(app: &mut App) {
    app.add_plugins((
        debug_normals::plugin,
        logs::plugin,
        editor_ui::plugin,
        gizmos::plugin,
        wireframe::plugin,
        debug_flags::plugin,
    ));
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        minimal_dev_tools_plugin,
        game::physics::devtools::plugin,
        // game::map::devtools::plugin,
        game::character_controller::devtools::plugin,
    ));
}
