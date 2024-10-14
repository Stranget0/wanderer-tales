use bevy_editor_pls::{
    editor_window::EditorWindow,
    egui::{self, TextureHandle},
    egui_dock::WindowState,
};
use bevy_inspector_egui::bevy_inspector;

use super::*;

// #[derive(Component)]
// struct ShaderMap;

#[derive(Component)]
pub struct DebugChunk;

pub fn map_devtools_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (
                log_terrain_changed.in_set(MapSystemSets::ChunkReload),
                clear_chunk_registry.in_set(MapSystemSets::ChunkReload),
                despawn_entities::<Chunk>.in_set(MapSystemSets::ChunkReload),
                spawn_chunks.in_set(MapSystemSets::ChunkMutate),
                // despawn_unregister_out_of_range_chunks.in_set(MapSystemSets::ChunkMutate),
                render_chunks.in_set(MapSystemSets::ChunkRender),
                // derender_chunks.in_set(MapSystemSets::ChunkRender),
            )
                .chain()
                .run_if(terrain_config_changed),
            debug_invisible_chunks.in_set(MapSystemSets::ChunkRender),
        ),
    );
}

fn log_terrain_changed() {
    info!("Terrain changed");
}

pub fn change_map_seed(mut terrain: ResMut<Terrain>) {
    terrain.noise_seed = terrain.noise_seed.wrapping_add(1);
    info!("Map seed: {}", terrain.noise_seed);
}

pub(super) fn terrain_config_changed(map_seed: Res<Terrain>) -> bool {
    map_seed.is_changed() && !map_seed.is_added()
}

fn clear_chunk_registry(mut chunk_manager: ResMut<ChunkManager>) {
    info!("Clearing chunk registry");
    chunk_manager.chunks.clear();
}

fn debug_invisible_chunks(
    chunks: Query<&ChunkPosition3, (With<Chunk>, Without<Handle<Mesh>>)>,
    mut gizmos: Gizmos,
) {
    for pos in chunks.iter() {
        gizmos.rect(
            pos.to_world_pos(),
            Quat::from_euler(EulerRot::XYZ, 90.0_f32.to_radians(), 0.0, 0.0),
            Vec2::splat(CHUNK_SIZE),
            tailwind::RED_500,
        );
    }
}

pub fn toggle_debug_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    center: Res<MapRenderCenter>,
    chunks: Query<&ChunkPosition3, With<Chunk>>,
    render_chunks: Query<Entity, With<DebugChunk>>,
    terrain: Res<Terrain>,
) {
    if !render_chunks.is_empty() {
        for entity in render_chunks.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let center = center.center.to_ivec().xz();
    let render_radius_squared =
        terrain.chunk_visibility_radius as i32 * terrain.chunk_visibility_radius as i32;

    for chunk_position in chunks.iter() {
        if !is_chunk_in_range(center, chunk_position, render_radius_squared) {
            continue;
        }
        let chunk_translation = chunk_position.to_2d().to_world_pos();
        let mesh = utils::primitives::create_subdivided_plane_smooth(
            terrain.chunk_subdivisions,
            CHUNK_SIZE,
            terrain.chunk_sampler(chunk_translation),
        );

        let transform =
            Transform::from_translation(chunk_position.to_world_pos() + (Vec3::Y * 10.0));
        let mesh_handle = asset_server.add(mesh);
        let material_handle = asset_server.add(StandardMaterial {
            base_color: tailwind::GRAY_400.into(),
            perceptual_roughness: 0.9,

            ..default()
        });

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform,
                ..default()
            },
            DebugChunk,
        ));
    }
}

struct TerrainConfigWindow;

struct TerrainConfigState {
    seed: u32,
    weigths: Vec<TerrainWeight>,
    weights_preview: Vec<egui::TextureHandle>,
}

const DEBUG_IMAGE_SIZE: usize = 250;

impl EditorWindow for TerrainConfigWindow {
    type State = TerrainConfigState;

    const NAME: &'static str = "Terrain Config";

    fn ui(
        world: &mut World,
        mut cx: bevy_editor_pls::editor_window::EditorWindowContext,
        ui: &mut bevy_editor_pls::egui::Ui,
    ) {
        let ctx = ui.ctx();
        let Some(state) = cx.state_mut::<TerrainConfigWindow>() else {
            return;
        };

        ui.label(Self::NAME);
        ui.collapsing("Details", |ui| {
            ui.horizontal(|ui| {
                ui.label("Preview:");
                for texture_handle in &state.weights_preview {
                    ui.image(texture_handle);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Seed:");
                if bevy_inspector::ui_for_value(&mut state.seed, ui, world) {
                    let mut terrain = world.get_resource_mut::<Terrain>().unwrap();
                    terrain.noise_seed = state.seed;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Weights:");
                if bevy_inspector::ui_for_value(&mut state.weigths, ui, world) {
                    let mut terrain = world.get_resource_mut::<Terrain>().unwrap();
                    terrain.noise_weights = state.weigths.clone();
                }
            });
        });
    }
}

impl Default for TerrainConfigState {
    fn default() -> Self {
        let terrain = Terrain::default();
        Self {
            seed: terrain.noise_seed,
            weigths: terrain.noise_weights,
            weights_preview: vec![],
        }
    }
}

// #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
// struct MapMaterialExtension {}
//
// impl MaterialExtension for MapMaterialExtension {
//     fn vertex_shader() -> ShaderRef {
//         "shaders/map.wgsl".into()
//     }
// }

// fn spawn_map_shader(mut commands: Commands, asset_server: Res<AssetServer>) {
//     let mesh = asset_server.add(utils::primitives::create_subdivided_plane(
//         CHUNK_SUBDIVISIONS * CHUNK_SPAWN_RADIUS as u32 * 2,
//         CHUNK_SIZE * CHUNK_SPAWN_RADIUS as f32 * 2.0,
//         |_x, _y| (0.0, Vec3::ZERO.into()),
//     ));
//
//     let base = StandardMaterial {
//         base_color: tailwind::GREEN_700.into(),
//         ..default()
//     };
//     let base = ExtendedMaterial {
//         base,
//         extension: MapMaterialExtension {},
//     };
//     let asset = ExtendedMaterial {
//         base,
//         extension: DebugNormalsMaterialExtension {},
//     };
//     let material = asset_server.add(asset);
//
//     commands.spawn((
//         ShaderMap,
//         MaterialMeshBundle {
//             mesh,
//             material,
//             ..default()
//         },
//     ));
// }
