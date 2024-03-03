use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use wanderer_tales::{debug::fps_counter::FPSPlugin, gameplay::plugin::GameplayPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EditorPlugin::default(),
            GameplayPlugin,
            FPSPlugin,
        ))
        .run();
}
