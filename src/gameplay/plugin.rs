use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec2, pbr::ExtendedMaterial, prelude::*};

use crate::{debug::local_position_gizmo::draw_local_gizmos, utils::MyExtension};

use super::{
    data_source_layer::{
        map::{
            components::*,
            resources::{HexToMapSourceEntity, SeedTable},
            DataSourceLayerPlugin, SourceLayerSet,
        },
        player::{events::CharacterMovedEvent, PlayerPlugin},
        utils::{hex_layout::HexLayout, layout_orientation::POINTY_TOP_ORIENTATION},
    },
    renderer::{
        camera::states::CameraMode,
        components::RenderGroup,
        renderers::{renderer_2d::Renderer2D, renderer_3d::Renderer3D},
        state::RendererState,
        RendererPlugin, RendererSet,
    },
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(RendererState::ThreeDimension)
            .insert_state(CameraMode::Follow)
            .insert_resource(SeedTable::default())
            .insert_resource(HexToMapSourceEntity::default())
            .register_type::<HexPosition>()
            .register_type::<Rotation>()
            .add_event::<CharacterMovedEvent>()
            .add_systems(Startup, initialize_map)
            .add_systems(
                Update,
                (
                    draw_local_gizmos::<Renderer3D>,
                    draw_local_gizmos::<Renderer2D>,
                ),
            )
            .configure_sets(
                Update,
                SourceLayerSet::PlayerInput.before(SourceLayerSet::Data),
            )
            .configure_sets(
                Update,
                RendererSet::RenderItems.before(SourceLayerSet::Data),
            )
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                DataSourceLayerPlugin,
                RendererPlugin,
                PlayerPlugin,
                MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyExtension>>::default(),
            ));
    }
}

fn initialize_map(mut commands: Commands) {
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
        Renderer3D::default(),
        gameplay_map_layout,
        RenderGroup::Gameplay3D,
        Name::new("GameplayMapLayout"),
    ));

    commands.spawn((
        SpatialBundle::default(),
        Renderer2D::default(),
        preview_map_layout,
        RenderGroup::PreviewMap2D,
        Name::new("PreviewMapLayout"),
    ));
}
