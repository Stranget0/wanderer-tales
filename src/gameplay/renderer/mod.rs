use bevy::prelude::*;

use crate::debug::switch_renderer::debug_switch_renderer;
use crate::utils::*;

use self::camera::bundles::*;
use self::camera::systems::{
    camera_follow, camera_rotation, camera_transform, camera_zoom, followed_rotation,
};
use self::components::{MaterialType, MeshType};
use self::renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D};
use self::state::RendererState;
use self::systems::*;

pub mod camera;
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

type ChangedRenderFilter = Or<(
    Added<MeshType>,
    Changed<MeshType>,
    Added<MaterialType>,
    Changed<MaterialType>,
)>;

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
            OnExit(RendererState::ThreeDimension),
            (
                hide_entity::<Renderer3D>,
                set_camera_state::<Camera3d, false>,
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
            OnExit(RendererState::TwoDimension),
            (
                set_camera_state::<Camera2d, false>,
                hide_entity::<Renderer2D>,
            ),
        )
        .add_systems(
            Update,
            (
                debug_switch_renderer,
                (
                    camera_zoom,
                    camera_rotation,
                    camera_follow::<Camera3d, Renderer3D>,
                    camera_follow::<Camera2d, Renderer2D>,
                    followed_rotation.before(RendererSet::RenderItems),
                )
                    .before(camera_transform),
                camera_transform,
                (
                    rotate_rendered_items::<Renderer2D>,
                    rotate_rendered_items::<Renderer3D>,
                )
                    .after(RendererSet::RenderItems),
                (
                    move_rendered_items::<Renderer2D>,
                    move_rendered_items::<Renderer3D>,
                    clean_render_items::<Renderer3D>,
                    clean_render_items::<Renderer2D>,
                    render_map_items::<Renderer3D, ChangedRenderFilter>,
                    render_map_items::<Renderer2D, ChangedRenderFilter>,
                )
                    .in_set(RendererSet::RenderItems),
            ),
        );
    }
}
