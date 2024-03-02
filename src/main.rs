use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use gameplay::plugin::GameplayPlugin;

mod gameplay;
pub mod global_state;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::default(), GameplayPlugin))
        .run();
}
