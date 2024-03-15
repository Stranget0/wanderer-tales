use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use self::{
    renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
    state::RendererState,
    systems::{
        despawn_map, fill_map, hide_entity, move_rendered_character, render_character, render_map,
        show_entity, synchronize_rendered_characters,
    },
};

use super::spawner::systems::spawn_map_data;

pub mod components;
pub mod debug;
pub mod events;
pub mod renderers;
pub mod state;
mod systems;
pub mod utils;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(RendererState::ThreeDimension),
            (
                fill_map::<PbrBundle, Renderer3D>,
                show_entity::<Renderer3D>,
                synchronize_rendered_characters::<Renderer3D>,
            ),
        )
        .add_systems(
            Update,
            (
                render_map::<PbrBundle, Renderer3D>.after(spawn_map_data),
                despawn_map::<Renderer3D>,
                render_character::<PbrBundle, Renderer3D>,
                move_rendered_character::<Renderer3D>,
            )
                .run_if(in_state(RendererState::ThreeDimension)),
        )
        .add_systems(
            OnExit(RendererState::ThreeDimension),
            hide_entity::<Renderer3D>,
        )
        .add_systems(
            OnEnter(RendererState::TwoDimension),
            (
                fill_map::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                show_entity::<Renderer2D>,
                synchronize_rendered_characters::<Renderer3D>,
            ),
        )
        .add_systems(
            Update,
            (
                render_map::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>.after(spawn_map_data),
                despawn_map::<Renderer2D>,
                render_character::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                move_rendered_character::<Renderer2D>,
            )
                .run_if(in_state(RendererState::TwoDimension)),
        )
        .add_systems(
            OnExit(RendererState::TwoDimension),
            hide_entity::<Renderer2D>,
        );
    }
}
