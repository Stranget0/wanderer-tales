pub mod events;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use crate::global_state::SceneState;

pub use events::*;

use self::systems::{
    add_hex_tile_offsets, clear_map_data, despawn_map_data, init_map_data, spawn_map_data,
};

pub struct MapSpawnerPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapSpawnerSet;

impl Plugin for MapSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_map_data,
                despawn_map_data,
                init_map_data,
                add_hex_tile_offsets
                    .after(spawn_map_data)
                    .after(despawn_map_data),
            )
                .in_set(MapSpawnerSet)
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnExit(SceneState::Menu), clear_map_data);
    }
}
