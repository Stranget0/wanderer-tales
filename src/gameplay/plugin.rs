use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec2, prelude::*};

use super::{
    map::{
        components::SourceLayout,
        renderer::{
            components::RenderGroup, events::RenderCharacterEvent, state::RendererState,
            RendererPlugin,
        },
        spawner::{
            resources::{HexToMapSourceEntity, SeedTable},
            MapAddEvent, MapSpawnerPlugin, MapSubEvent,
        },
        utils::{hex_layout::HexLayout, layout_orientation::POINTY_TOP_ORIENTATION},
    },
    player::{
        events::{CharacterMovedEvent, PlayerWithSightSpawnedEvent},
        PlayerPlugin,
    },
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        let source_layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(1.0, 1.0),
            origin: vec2(0.0, 0.0),
        };
        let preview_map_layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(16.0, 16.0),
            origin: vec2(0.0, 0.0),
        };

        let gameplay_map_layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(1.0, 1.0),
            origin: vec2(0.0, 0.0),
        };

        app.world.spawn((source_layout, SourceLayout));

        app.world.spawn((
            gameplay_map_layout,
            RenderGroup::Gameplay3D,
            SpatialBundle::default(),
        ));

        app.world.spawn((
            preview_map_layout,
            RenderGroup::PreviewMap2D,
            SpatialBundle::default(),
        ));

        app.insert_state(RendererState::TwoDimension)
            // .init_state::<RendererState>()
            .insert_resource(SeedTable::default())
            .insert_resource(HexToMapSourceEntity::default())
            .add_event::<MapAddEvent>()
            .add_event::<MapSubEvent>()
            .add_event::<RenderCharacterEvent>()
            .add_event::<CharacterMovedEvent>()
            .add_event::<PlayerWithSightSpawnedEvent>()
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                MapSpawnerPlugin,
                RendererPlugin,
                PlayerPlugin,
            ));
    }
}
