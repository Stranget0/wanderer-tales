#[cfg(feature = "dev")]
pub mod dev_tools;
pub mod game;
mod screen;
mod ui;

pub mod extenstions;
pub mod utils;
pub mod wgsl_keys;

use bevy::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    log::LogPlugin,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};

pub mod prelude {
    pub use super::extenstions::*;
    pub use super::utils;
    pub use super::utils::ecs::*;
    pub use super::wgsl_keys::*;
    pub use crate::game::*;
    pub use crate::screen::GameState;
    pub use crate::GameSet;
    pub use bevy::color::palettes::tailwind;
    pub use bevy::input::common_conditions::*;
    pub use bevy::math::*;
    pub use bevy::prelude::*;
    pub use bevy::utils::hashbrown;
    pub use itertools::Itertools;

    #[cfg(feature = "dev")]
    pub use crate::dev_tools::*;
}

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=debug";
pub fn default_plugins(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Wanderer Tales".to_string(),
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(AudioPlugin {
                global_volume: GlobalVolume {
                    volume: Volume::new(0.3),
                },
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    #[cfg(feature = "dev")]
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                filter: LOG_FILTER.to_string(),
                level: bevy::log::Level::INFO,

                ..default()
            }),
    );
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (
                GameSet::TickTimers,
                GameSet::RecordInput,
                GameSet::UpdateDataLayer,
                GameSet::UpdateApply,
                GameSet::Update,
                GameSet::PostUpdate,
            )
                .chain(),
        );

        app.add_plugins(default_plugins);

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera_ui);

        // app.add_plugins(RenderStatePlugin::new());

        // Add other plugins.
        app.add_plugins((game::plugin, screen::plugin, ui::plugin));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameSet {
    TickTimers,
    RecordInput,
    UpdateDataLayer,
    UpdateApply,
    Update,
    PostUpdate,
}

fn spawn_camera_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("UI Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}
