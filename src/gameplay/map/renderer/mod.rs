use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    debug::switch_renderer::debug_switch_renderer,
    utils::{hide_entity, spawn_default_with_parent},
};

use self::{
    bundles::{Game2DCameraBundle, Game3DCameraBundle},
    renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
    state::RendererState,
    systems::{
        camera_look_around, camera_update, camera_zoom, clean_render_items, move_rendered_items,
        remove_moving_render_items, render_map_items, render_static_map_items, set_camera_state,
        show_entity,
    },
};

mod bundles;
pub mod components;
pub mod debug;
pub mod renderers;
pub mod state;
mod systems;

pub struct RendererPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RendererSet {
    LayoutInit,
    RenderItems,
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_default_with_parent::<Game3DCameraBundle, With<Renderer3D>>,
                spawn_default_with_parent::<Game2DCameraBundle, With<Renderer2D>>,
            )
                .in_set(RendererSet::LayoutInit),
        )
        .add_systems(
            OnEnter(RendererState::ThreeDimension),
            (
                show_entity::<Renderer3D>,
                set_camera_state::<Camera3d, true>,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    render_static_map_items::<PbrBundle, Renderer3D>,
                    render_map_items::<PbrBundle, Renderer3D>,
                    clean_render_items::<Renderer3D>,
                    move_rendered_items::<Renderer3D>,
                )
                    .in_set(RendererSet::RenderItems),
                camera_update::<Renderer3D>.after(camera_look_around),
            )
                .run_if(in_state(RendererState::ThreeDimension)),
        )
        .add_systems(
            OnExit(RendererState::ThreeDimension),
            (
                hide_entity::<Renderer3D>,
                set_camera_state::<Camera3d, false>,
                remove_moving_render_items::<Renderer3D>.in_set(RendererSet::RenderItems),
            ),
        )
        .add_systems(
            OnEnter(RendererState::TwoDimension),
            (
                show_entity::<Renderer2D>,
                set_camera_state::<Camera2d, true>,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    render_static_map_items::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                    render_map_items::<MaterialMesh2dBundle<ColorMaterial>, Renderer2D>,
                    clean_render_items::<Renderer2D>,
                    move_rendered_items::<Renderer2D>,
                )
                    .in_set(RendererSet::RenderItems),
                camera_update::<Renderer2D>.after(camera_look_around),
            )
                .run_if(in_state(RendererState::TwoDimension)),
        )
        .add_systems(
            OnExit(RendererState::TwoDimension),
            (
                set_camera_state::<Camera2d, false>,
                hide_entity::<Renderer2D>,
                remove_moving_render_items::<Renderer2D>,
            ),
        )
        .add_systems(
            Update,
            (debug_switch_renderer, camera_look_around, camera_zoom),
        );
    }
}
