use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    debug::switch_renderer::debug_switch_renderer,
    gameplay::player::systems::spawn_player,
    util_systems::{despawn_with_parent, hide_entity, spawn_default_with_parent},
};

use self::{
    bundles::{Game2DCameraBundle, Game3DCameraBundle},
    renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
    state::RendererState,
    systems::{
        camera_follow, clean_render_items, move_rendered_items, remove_moving_render_items,
        render_map_items, render_static_map_items, show_entity,
    },
};

use super::spawner::systems::{despawn_map_data, spawn_map_data};

mod bundles;
pub mod components;
pub mod debug;
pub mod renderers;
pub mod state;
mod systems;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(RendererState::ThreeDimension),
            (
                show_entity::<Renderer3D>,
                // synchronize_rendered_characters::<Renderer3D>,
                spawn_default_with_parent::<Game3DCameraBundle, With<Renderer3D>>,
            ),
        )
        .add_systems(
            Update,
            (
                render_static_map_items::<PbrBundle, Renderer3D>.before(spawn_map_data),
                render_map_items::<PbrBundle, Renderer3D>.before(spawn_player),
                clean_render_items::<Renderer3D>.before(despawn_map_data),
                move_rendered_items::<Renderer3D>,
                camera_follow::<Renderer3D>,
            )
                .run_if(in_state(RendererState::ThreeDimension)),
        )
        .add_systems(
            OnExit(RendererState::ThreeDimension),
            (
                hide_entity::<Renderer3D>,
                despawn_with_parent::<With<Camera3d>>,
                remove_moving_render_items::<Renderer3D>,
            ),
        )
        .add_systems(
            OnEnter(RendererState::TwoDimension),
            (
                show_entity::<Renderer2D>,
                // synchronize_rendered_characters::<Renderer2D>,
                spawn_default_with_parent::<Game2DCameraBundle, With<Renderer2D>>,
            ),
        )
        .add_systems(
            Update,
            (
                render_static_map_items::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>
                    .before(spawn_map_data),
                render_map_items::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>
                    .before(spawn_player),
                clean_render_items::<Renderer2D>.before(despawn_map_data),
                move_rendered_items::<Renderer2D>,
                camera_follow::<Renderer2D>,
            )
                .run_if(in_state(RendererState::TwoDimension)),
        )
        .add_systems(
            OnExit(RendererState::TwoDimension),
            (
                despawn_with_parent::<With<Camera2d>>,
                hide_entity::<Renderer2D>,
                remove_moving_render_items::<Renderer2D>,
            ),
        )
        .add_systems(Update, debug_switch_renderer);
    }
}
