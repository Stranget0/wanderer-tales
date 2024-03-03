use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use crate::global_state::SceneState;

use super::{
    camera::spawn_camera,
    map::{
        events::{MapAddEvent, MoveSightEvent},
        renderer::{
            events::RenderPointEvent,
            rendered_2d::{render_map, render_point},
        },
        spawner::{despawn_map_data, spawn_layout, spawn_map_data},
    },
    player::{events::WSADEvent, move_2d_handle, move_interaction, spawn_player},
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // #[cfg(debug_assertions)]
        // {
        //     use bevy::diagnostic::LogDiagnosticsPlugin;
        //     app.add_plugins(LogDiagnosticsPlugin::default());
        // }

        app.init_state::<SceneState>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_event::<MoveSightEvent>()
            .add_event::<RenderPointEvent>()
            .add_event::<WSADEvent>()
            .add_event::<MapAddEvent>()
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                (spawn_layout, spawn_player.after(spawn_layout), spawn_camera),
            )
            .add_systems(
                Update,
                (
                    spawn_map_data,
                    render_map.after(spawn_map_data),
                    render_point,
                    move_interaction,
                    move_2d_handle,
                ),
            )
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}
