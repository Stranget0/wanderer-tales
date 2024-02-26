use bevy::prelude::*;

use crate::global_state::SceneState;

use super::{
    renderer::rendered_2d::render_map,
    spawner::{despawn_map_data, spawn_layout, spawn_map_data},
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                ((spawn_layout, spawn_map_data, render_map).chain(),),
            )
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}
