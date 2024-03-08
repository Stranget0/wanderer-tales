use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec2, prelude::*, utils::info};

use super::{
    map::{
        renderer::{events::RenderCharacterEvent, state::RendererState, RendererPlugin},
        spawner::{
            resources::{MapData, SeedTable},
            MapAddEvent, MapSpawnerPlugin, MoveSightEvent,
        },
        utils::{hex_layout::HexLayout, layout_orientation::POINTY_TOP_ORIENTATION},
    },
    player::PlayerPlugin,
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(RendererState::TwoDimension)
            // .init_state::<RendererState>()
            .insert_resource(SeedTable::default())
            .insert_resource(MapData::default())
            .add_event::<MoveSightEvent>()
            .add_event::<MapAddEvent>()
            .add_event::<RenderCharacterEvent>()
            .add_systems(Startup, spawn_layout)
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                MapSpawnerPlugin,
                RendererPlugin,
                PlayerPlugin,
            ));
    }
}

pub fn spawn_layout(mut commands: Commands) {
    let layout: HexLayout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(5.0, 5.0),
        origin: vec2(0.0, 0.0),
    };

    commands.spawn((
        layout,
        SpatialBundle {
            ..Default::default()
        },
    ));
}
