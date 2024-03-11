pub mod events;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use crate::global_state::SceneState;

pub use events::*;

use self::systems::{clear_map_data, despawn_map_data, init_map_data, spawn_map_data};

pub struct MapSpawnerPlugin;

impl Plugin for MapSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_map_data, despawn_map_data, init_map_data).run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnExit(SceneState::Menu), clear_map_data);
    }
}
