use self::{
    resources::{init_materials_store, init_meshes_store, MaterialStore2d, MeshesStore2d},
    systems::{delete_maps, despawn_camera, render_character, render_map, spawn_camera},
};
use crate::gameplay::map::spawner::systems::spawn_map_data;
use bevy::prelude::*;

use super::state::RendererState;

pub mod resources;
pub mod systems;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshesStore2d::default())
            .insert_resource(MaterialStore2d::default())
            .add_systems(
                OnEnter(RendererState::TwoDimension),
                (init_meshes_store, init_materials_store, spawn_camera),
            )
            .add_systems(
                Update,
                (render_map.after(spawn_map_data), render_character)
                    .run_if(in_state(RendererState::TwoDimension)),
            )
            .add_systems(
                OnExit(RendererState::TwoDimension),
                (delete_maps, despawn_camera),
            );
    }
}
