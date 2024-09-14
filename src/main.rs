// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_dexterous_developer::*;
use wanderer_tales::AppPlugin;

reloadable_main!((initial_plugins) {
    App::new()
        .add_plugins(
            initial_plugins.initialize::<DefaultPlugins>()
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
                    filter: "wgpu=error,naga=warn,bevy_ecs=debug".to_string(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_plugins(AppPlugin)
        .run()
});
