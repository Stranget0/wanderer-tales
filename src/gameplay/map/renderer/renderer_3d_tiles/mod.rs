use self::{
    resources::{init_materials_store, init_meshes_store, MaterialStore3d, MeshesStore3d},
    systems::{despawn_camera, free_map, render_character, render_map, spawn_camera},
};
use crate::gameplay::{map::spawner::systems::spawn_map_data, plugin::spawn_layout};
use bevy::prelude::*;

use super::state::RendererState;

pub mod resources;
pub mod systems;

pub struct Renderer3DPlugin;

impl Plugin for Renderer3DPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshesStore3d::default())
            .insert_resource(MaterialStore3d::default())
            .add_systems(
                OnEnter(RendererState::ThreeDimension),
                (
                    init_meshes_store.after(spawn_layout),
                    init_materials_store,
                    spawn_camera,
                ),
            )
            .add_systems(
                Update,
                (render_map.after(spawn_map_data), render_character)
                    .run_if(in_state(RendererState::ThreeDimension)),
            )
            .add_systems(
                OnExit(RendererState::ThreeDimension),
                (free_map, despawn_camera),
            );
    }
}
