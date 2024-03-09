use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec2, prelude::*, utils::info};

use super::{
    map::{
        components::{MapContent, MapDisplay, WithPlayerRender},
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
        // let layout = HexLayout {
        //     orientation: POINTY_TOP_ORIENTATION,
        //     size: vec2(64.0, 64.0),
        //     origin: vec2(0.0, 0.0),
        // };
        let layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(1.0, 1.0),
            origin: vec2(0.0, 0.0),
        };

        let map_display = app
            .world
            .spawn((MapDisplay, WithPlayerRender, SpatialBundle::default()))
            .id();

        app.world
            .spawn((MapContent, SpatialBundle::default()))
            .add_child(map_display);

        app.insert_state(RendererState::ThreeDimension)
            // .init_state::<RendererState>()
            .insert_resource(SeedTable::default())
            .insert_resource(MapData::default())
            .insert_resource(layout)
            .add_event::<MoveSightEvent>()
            .add_event::<MapAddEvent>()
            .add_event::<RenderCharacterEvent>()
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                MapSpawnerPlugin,
                RendererPlugin,
                PlayerPlugin,
            ));
    }
}
