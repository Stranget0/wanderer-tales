use bevy::prelude::*;

use crate::{gameplay::map::utils::hex_vector::HexVector, global_state::SceneState};

use super::map::{
    events::{MapAddEvent, MoveMapOriginEvent},
    renderer::rendered_2d::render_map,
    spawner::{despawn_map_data, spawn_layout, spawn_map_data},
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_event::<MoveMapOriginEvent>()
            .add_event::<MapAddEvent>()
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                (spawn_layout, test_map),
            )
            .add_systems(Update, (spawn_map_data, render_map))
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}

fn test_map(mut move_map_origin: EventWriter<MoveMapOriginEvent>) {
    println!("SEND MOVE MAP ORIGIN");
    move_map_origin.send(MoveMapOriginEvent(HexVector::new(0, 0, 0)));
}