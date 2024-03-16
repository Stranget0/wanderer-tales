use bevy::prelude::*;
use wanderer_tales::{
    debug::fps_counter::FPSPlugin, gameplay::plugin::GameplayPlugin, global_state::SceneState,
};

fn main() {
    App::new()
        // .init_state::<SceneState>()
        .insert_state(SceneState::Game)
        .add_plugins((
            DefaultPlugins,
            // EditorPlugin::default(),
            GameplayPlugin,
            FPSPlugin,
        ))
        .run();
}
