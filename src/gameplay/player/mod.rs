use bevy::prelude::*;

use crate::global_state::SceneState;

use self::{
    events::WSADEvent,
    systems::{move_2d_handle, move_interaction, spawn_player},
};

use super::plugin::spawn_layout;

pub mod components;
pub mod events;
pub mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WSADEvent>()
            .add_systems(OnEnter(SceneState::Game), spawn_player.after(spawn_layout))
            .add_systems(Update, (move_interaction, move_2d_handle));
    }
}
