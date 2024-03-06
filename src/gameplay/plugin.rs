use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use crate::global_state::SceneState;

use super::{
    camera::spawn_camera,
    map::{
        events::{MapAddEvent, MoveSightEvent},
        renderer::{
            events::RenderCharacter,
            renderer_2d::{
                render_map, render_point,
                resources::{init_materials_store, init_meshes_store, MaterialStore, MeshesStore},
            },
        },
        spawner::{
            despawn_map_data,
            resources::{MapData, SeedTable},
            spawn_layout, spawn_map_data,
        },
    },
    player::{events::WSADEvent, move_2d_handle, move_interaction, spawn_player},
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_event::<MoveSightEvent>()
            .add_event::<RenderCharacter>()
            .add_event::<WSADEvent>()
            .add_event::<MapAddEvent>()
            .insert_resource(MapData::default())
            .insert_resource(SeedTable::default())
            .insert_resource(MeshesStore::default())
            .insert_resource(MaterialStore::default())
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                (
                    spawn_layout,
                    spawn_player.after(spawn_layout),
                    init_meshes_store.after(spawn_layout),
                    init_materials_store,
                    spawn_camera,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_map_data,
                    render_map.after(spawn_map_data),
                    despawn_map_data,
                    render_point,
                    move_interaction,
                    move_2d_handle,
                ),
            )
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}
