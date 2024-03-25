use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec2, prelude::*};

use crate::debug::local_position_gizmo::draw_local_gizmos;

use super::{
    components::HexPosition,
    map::{
        components::SourceLayout,
        renderer::{
            components::RenderGroup,
            renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
            state::RendererState,
            RendererPlugin, RendererSet,
        },
        spawner::{
            resources::{HexToMapSourceEntity, SeedTable},
            MapAddEvent, MapSpawnerPlugin, MapSpawnerSet, MapSubEvent,
        },
        utils::{hex_layout::HexLayout, layout_orientation::POINTY_TOP_ORIENTATION},
    },
    player::{events::CharacterMovedEvent, PlayerPlugin},
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(RendererState::ThreeDimension)
            .insert_resource(SeedTable::default())
            .insert_resource(HexToMapSourceEntity::default())
            .register_type::<HexPosition>()
            .add_event::<MapAddEvent>()
            .add_event::<MapSubEvent>()
            .add_event::<CharacterMovedEvent>()
            .add_systems(Startup, initialize_map)
            .add_systems(Update, draw_local_gizmos)
            .configure_sets(Update, RendererSet::RenderItems.before(MapSpawnerSet))
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                MapSpawnerPlugin,
                RendererPlugin,
                PlayerPlugin,
            ));
    }
}

fn initialize_map(
    mut commands: Commands,
    mut materials_2d: ResMut<Assets<ColorMaterial>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let source_layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(1.0, 1.0),
        origin: vec2(0.0, 0.0),
    };

    let preview_map_layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(8.0, 8.0),
        origin: vec2(0.0, 0.0),
    };

    let gameplay_map_layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(1.0, 1.0),
        origin: vec2(0.0, 0.0),
    };

    commands.spawn((source_layout, SourceLayout, Name::new("MapSourceData")));

    commands.spawn((
        SpatialBundle::default(),
        Renderer3D::new(
            &gameplay_map_layout,
            &mut materials_3d,
            &mut images,
            &mut meshes,
        ),
        gameplay_map_layout,
        RenderGroup::Gameplay3D,
        Name::new("GameplayMapLayout"),
    ));

    commands.spawn((
        SpatialBundle::default(),
        Renderer2D::new(&preview_map_layout, &mut materials_2d, &mut meshes),
        preview_map_layout,
        RenderGroup::PreviewMap2D,
        Name::new("PreviewMapLayout"),
    ));
}
