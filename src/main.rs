use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use debug::fps_counter::FPSPlugin;
use gameplay::plugin::GameplayPlugin;

pub mod debug;
mod gameplay;
pub mod global_state;

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
