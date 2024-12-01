// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::app::App;
use wanderer_tales::AppPlugin;

fn main() {
    App::new().add_plugins(AppPlugin).run();
}
