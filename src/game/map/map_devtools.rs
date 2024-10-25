use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_inspector;
use plugin::*;

use super::*;

// #[derive(Component)]
// struct ShaderMap;

const DEBUG_IMAGE_SIZE: usize = 75;

#[derive(Component)]
pub struct DebugChunk;

#[derive(Resource)]
pub struct EditorTerrainState {
    weights: Vec<TerrainWeight>,
    weights_preview: Vec<TerrainPreview>,
    weights_visibility: Vec<bool>,
    seed: u32,
    preview_scale: f32,
}

#[derive(Clone)]
struct TerrainPreview {
    individual: egui::TextureHandle,
    combined: egui::TextureHandle,
    reference: egui::TextureHandle,

    erosion_individual: egui::TextureHandle,
    erosion_combined: egui::TextureHandle,
    erosion_reference: egui::TextureHandle,
}

pub fn map_devtools_plugin(app: &mut App) {
    app.init_resource::<EditorTerrainState>().add_systems(
        Update,
        (
            (
                synchronize_terrain_ui.in_set(MapSystemSets::ChunkReload),
                log_terrain_changed.in_set(MapSystemSets::ChunkReload),
                clear_chunk_registry.in_set(MapSystemSets::ChunkReload),
                despawn_entities::<Chunk>.in_set(MapSystemSets::ChunkReload),
                spawn_chunks.in_set(MapSystemSets::ChunkMutate),
                // despawn_unregister_out_of_range_chunks.in_set(MapSystemSets::ChunkMutate),
                render_chunks.in_set(MapSystemSets::ChunkRender),
                // derender_chunks.in_set(MapSystemSets::ChunkRender),
            )
                .chain()
                .run_if(resource_changed::<Terrain>),
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
        let mesh = utils::primitives::create_subdivided_plane(
            terrain.chunk_subdivisions,
            CHUNK_SIZE,
            terrain.chunk_sampler_estimate(chunk_translation),
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

fn synchronize_terrain_ui(
    ui_context: Query<&bevy_inspector_egui::bevy_egui::EguiContext, With<PrimaryWindow>>,
    terrain: Res<Terrain>,
    mut terrain_ui: ResMut<EditorTerrainState>,
) {
    let Ok(egui_context) = ui_context.get_single() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    terrain_ui.synchronize_with_terrain(&terrain, egui_context.get_mut());
}

impl Default for EditorTerrainState {
    fn default() -> Self {
        Self {
            weights: Vec::new(),
            weights_preview: Vec::new(),
            seed: 0,
            preview_scale: 25.0,
        }
    }
}

pub struct EditorTerrain {}

impl EditorTerrainState {
    fn create_preview_texture(
        &self,
        ctx: &egui::Context,
        sample: impl Fn(Vec2) -> f32,
    ) -> egui::TextureHandle {
        let mut image = egui::ColorImage {
            size: [DEBUG_IMAGE_SIZE, DEBUG_IMAGE_SIZE],
            pixels: vec![egui::Color32::BLACK; DEBUG_IMAGE_SIZE * DEBUG_IMAGE_SIZE],
        };
        for x in 0..DEBUG_IMAGE_SIZE {
            for y in 0..DEBUG_IMAGE_SIZE {
                let pos = vec2(x as f32 * self.preview_scale, y as f32 * self.preview_scale);
                let value = sample(pos) * 255.0;

                let color = egui::Color32::from_gray(value as u8);
                image.pixels[x + y * DEBUG_IMAGE_SIZE] = color;
            }
        }

        ctx.load_texture(
            "debug",
            image,
            egui::TextureOptions {
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                ..default()
            },
        )
    }

    fn synchronize_with_terrain(&mut self, terrain: &Terrain, ctx: &mut egui::Context) {
        self.weights_preview.clear();
        self.weights = terrain.noise_weights.clone();
        self.weights_visibility = vec![true; self.weights.len()];
        let highest_amplitude = self
            .weights
            .iter()
            .max_by(|a, b| a.amplitude.partial_cmp(&b.amplitude).unwrap())
            .map(|w| w.amplitude)
            .unwrap_or(1.0);

        let hasher = noise::PcgHasher::from_seed(terrain.noise_seed);
        let mut passed_weights = Vec::with_capacity(self.weights.len());

        for weight in &self.weights {
            passed_weights.push(weight);

            let individual = self.create_preview_texture(ctx, |pos| {
                weight.sample_erosion_base(&hasher, pos, 0.0).0.value / highest_amplitude
            });

            let combined = self.create_preview_texture(ctx, |pos| {
                TerrainWeight::sample_many(
                    hasher.clone(),
                    pos,
                    passed_weights.iter().map(|w| w as &_),
                )
                .0
                .value
                    / highest_amplitude
            });

            let reference = self.create_preview_texture(ctx, |pos| {
                TerrainWeight::sample_many_reference(
                    hasher.clone(),
                    pos,
                    passed_weights.iter().map(|w| w as &_),
                )
                .0
                .value
                    / highest_amplitude
            });

            let erosion_individual = self
                .create_preview_texture(ctx, |pos| weight.sample_erosion_base(&hasher, pos, 0.0).1);

            let erosion_combined = self.create_preview_texture(ctx, |pos| {
                TerrainWeight::sample_many(
                    hasher.clone(),
                    pos,
                    passed_weights.iter().map(|w| w as &_),
                )
                .1
            });

            let erosion_reference = self.create_preview_texture(ctx, |pos| {
                TerrainWeight::sample_many_reference(
                    hasher.clone(),
                    pos,
                    passed_weights.iter().map(|w| w as &_),
                )
                .1
            });

            self.weights_preview.push(TerrainPreview {
                individual,
                combined,
                reference,

                erosion_individual,
                erosion_combined,
                erosion_reference,
            });
        }
    }
}

impl EditorDock for EditorTerrain {
    fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
        world.resource_scope::<EditorTerrainState, _>(|world, mut state| {
            let is_focused = ui.memory(|m| m.clone()).focused().is_some();

            ui.collapsing("Preview", |ui| {
                for weight_preview in state.weights_preview.iter() {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Individual");
                            ui.image(&weight_preview.individual);
                            ui.image(&weight_preview.erosion_individual);
                        });
                        ui.vertical(|ui| {
                            ui.label("Combined");
                            ui.image(&weight_preview.combined);
                            ui.image(&weight_preview.erosion_combined);
                        });
                        ui.vertical(|ui| {
                            ui.label("Reference");
                            ui.image(&weight_preview.reference);
                            ui.image(&weight_preview.erosion_reference);
                        });
                    });
                    ui.separator();
                    ui.spacing();
                }
            });

            ui.collapsing("Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Seed:");
                    if bevy_inspector::ui_for_value(&mut state.seed, ui, world) {
                        let mut terrain = world.get_resource_mut::<Terrain>().unwrap();
                        terrain.noise_seed = state.seed;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Weights:");
                    if bevy_inspector::ui_for_value(&mut state.weights, ui, world) {
                        let mut terrain = world.get_resource_mut::<Terrain>().unwrap();
                        terrain.noise_weights = state.weights.clone();
                    }
                });
            });
        });
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
