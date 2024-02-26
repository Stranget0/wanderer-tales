use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use features::{camera::spawn_camera, map::plugin::MapPlugin};

mod features;
pub mod global_state;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::default(), MapPlugin))
        .add_systems(Startup, spawn_camera)
        .run();
}
