use bevy::prelude::*;

use crate::global_state::SceneState;

use self::{
    events::WSADEvent,
    systems::{move_2d_handle, move_interaction, rotate_controlled_source, spawn_player},
};

use super::map::SourceLayerSet;

pub mod components;
pub mod events;
pub mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WSADEvent>()
            .add_systems(OnEnter(SceneState::Game), spawn_player)
            .add_systems(
                Update,
                (
                    move_interaction.in_set(SourceLayerSet::PlayerInput),
                    (move_2d_handle, rotate_controlled_source).in_set(SourceLayerSet::Data),
                ),
            );
    }
}
