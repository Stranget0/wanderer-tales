use self::{
    resources::{
        init_materials_store, init_meshes_store, MaterialStore3d, MeshesStore3d,
        SourceToRenderStore3d,
    },
    systems::{
        delete_render_groups, despawn_camera, despawn_map, move_rendered_character,
        render_character, render_map, spawn_camera,
    },
};
use crate::gameplay::map::spawner::systems::spawn_map_data;
use bevy::prelude::*;

use super::state::RendererState;

pub mod mesh;
pub mod resources;
pub mod systems;

pub struct Renderer3DPlugin;

impl Plugin for Renderer3DPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshesStore3d::default())
            .insert_resource(MaterialStore3d::default())
            .insert_resource(SourceToRenderStore3d::default())
            .add_systems(
                OnEnter(RendererState::ThreeDimension),
                (init_meshes_store, init_materials_store, spawn_camera),
            )
            .add_systems(
                Update,
                (
                    render_map.after(spawn_map_data),
                    despawn_map,
                    render_character,
                    move_rendered_character,
                )
                    .run_if(in_state(RendererState::ThreeDimension)),
            )
            .add_systems(
                OnExit(RendererState::ThreeDimension),
                (delete_render_groups, despawn_camera),
            );
    }
}