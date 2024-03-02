use bevy::prelude::*;

use crate::global_state::SceneState;

use super::{
    map::{
        events::{MapAddEvent, MoveMapOriginEvent},
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
        app.init_state::<SceneState>()
            .add_event::<MoveMapOriginEvent>()
            .add_event::<RenderPointEvent>()
            .add_event::<WSADEvent>()
            .add_event::<MapAddEvent>()
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                spawn_layout.before(spawn_player),
            )
            .add_systems(
                Update,
                (
                    spawn_map_data,
                    render_map,
                    // render_point,
                    // move_interaction,
                    // move_2d_handle,
                ),
            )
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}

// fn test_map(mut move_map_origin: EventWriter<MoveMapOriginEvent>) {
//     println!("SEND MOVE MAP ORIGIN");
//     move_map_origin.send(MoveMapOriginEvent(HexVector::new(0, 0, 0)));
// }
