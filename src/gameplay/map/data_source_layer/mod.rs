pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
use self::systems::*;
use crate::global_state::SceneState;
use bevy::prelude::*;
pub use events::*;

pub struct DataSourceLayerPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceLayerSet {
    PlayerInput,
    Data,
}

impl Plugin for DataSourceLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_map_data,
                despawn_map_data,
                fill_map_data_on_sight,
                add_hex_tile_offsets
                    .after(spawn_map_data)
                    .after(despawn_map_data),
            )
                .in_set(SourceLayerSet::Data)
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnExit(SceneState::Menu), clear_map_data);
    }
}
