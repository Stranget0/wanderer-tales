use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    debug::switch_renderer::debug_switch_renderer,
    util_systems::{despawn_with_parent, hide_entity, spawn_default_with_parent},
};

use self::{
    bundles::{Game2DCameraBundle, Game3DCameraBundle},
    renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
    state::RendererState,
    systems::{
        camera_follow, despawn_map, empty_map, fill_map, move_rendered_character, render_character,
        render_map, show_entity, synchronize_rendered_characters,
    },
};

use super::spawner::systems::{despawn_map_data, spawn_map_data};

mod bundles;
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
                // synchronize_rendered_characters::<Renderer3D>,
                spawn_default_with_parent::<Game3DCameraBundle, With<Renderer3D>>,
            ),
        )
        .add_systems(
            Update,
            (
                render_map::<PbrBundle, Renderer3D>,
                despawn_map::<Renderer3D>.after(despawn_map_data),
                render_character::<PbrBundle, Renderer3D>,
                move_rendered_character::<Renderer3D>,
                camera_follow::<Renderer3D>,
            )
                .run_if(in_state(RendererState::ThreeDimension)),
        )
        .add_systems(
            OnExit(RendererState::ThreeDimension),
            (
                hide_entity::<Renderer3D>,
                despawn_with_parent::<With<Camera3d>>,
                empty_map::<Renderer3D>,
            ),
        )
        .add_systems(
            OnEnter(RendererState::TwoDimension),
            (
                fill_map::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                show_entity::<Renderer2D>,
                // synchronize_rendered_characters::<Renderer2D>,
                spawn_default_with_parent::<Game2DCameraBundle, With<Renderer2D>>,
            ),
        )
        .add_systems(
            Update,
            (
                render_map::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>.after(spawn_map_data),
                despawn_map::<Renderer2D>.after(despawn_map_data),
                render_character::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                move_rendered_character::<Renderer2D>,
                camera_follow::<Renderer2D>,
            )
                .run_if(in_state(RendererState::TwoDimension)),
        )
        .add_systems(
            OnExit(RendererState::TwoDimension),
            (
                despawn_with_parent::<With<Camera2d>>,
                hide_entity::<Renderer2D>,
                empty_map::<Renderer2D>,
            ),
        )
        .add_systems(Update, debug_switch_renderer);
    }
}
