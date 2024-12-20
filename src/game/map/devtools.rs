use super::*;
use crate::dev_tools::editor_ui::EditorDock;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_inspector;

enum Flags {
    DebugUnrenderedChunks,
    DisplayWorldGrid,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Terrain"
    }
    fn as_str(&self) -> &'static str {
        match self {
            Flags::DebugUnrenderedChunks => "Debug unrendered chunks",
            Flags::DisplayWorldGrid => "Display world grid",
        }
    }
}

// #[derive(Component)]
// struct ShaderMap;

const DEBUG_IMAGE_SIZE: usize = 100;

#[derive(Component)]
pub struct DebugChunk;

#[derive(Resource)]
pub struct EditorTerrainState {
    manual_has_changed: bool,
    weights: Vec<(bool, TerrainWeight)>,
    seed: u32,
    chunk_subdivisions: u32,
    // These are in chunks
    chunk_spawn_radius: u8,
    chunk_visibility_radius: u8,
}

#[derive(Resource)]
pub struct EditorTerrainImages {
    weights_preview: Vec<TerrainPreview>,
    preview_scale: f32,
    manual_has_changed: bool,
}
impl Default for EditorTerrainImages {
    fn default() -> Self {
        Self {
            weights_preview: Vec::new(),
            preview_scale: 25.0,
            manual_has_changed: false,
        }
    }
}

#[derive(Clone)]
struct TerrainPreview {
    individual: egui::TextureHandle,
    combined: egui::TextureHandle,

    erosion_individual: egui::TextureHandle,
    erosion_combined: egui::TextureHandle,
}

pub fn plugin(app: &mut App) {
    register_debug_flags(
        app,
        vec![Flags::DisplayWorldGrid, Flags::DebugUnrenderedChunks],
    );

    app.init_resource::<EditorTerrainImages>()
        .init_resource::<EditorTerrainState>()
        .add_systems(OnEnter(GameState::Playing), update_terrain_previews)
        .add_systems(
            Update,
            (
                display_world_grid
                    .in_set(ChunkSystemSet::Render)
                    .run_if(debug_flag_enabled(&Flags::DisplayWorldGrid)),
                debug_invisible_chunks
                    .in_set(ChunkSystemSet::Render)
                    .run_if(debug_flag_enabled(&Flags::DebugUnrenderedChunks)),
                sync_terrain_with_ui
                    .in_set(ChunkSystemSet::ChunkReload)
                    .run_if(editor_terrain_changed),
                update_terrain_previews
                    .in_set(ChunkSystemSet::ChunkReload)
                    .run_if(
                        editor_terrain_changed
                            .or_else(editor_terrain_previews_changed)
                            .or_else(render_center_changed),
                    ),
                unflag_manual_terrain_change
                    .in_set(ChunkSystemSet::Render)
                    .run_if(editor_terrain_changed),
                unflag_manual_terrain_previews_change
                    .in_set(ChunkSystemSet::Render)
                    .run_if(editor_terrain_previews_changed),
                (
                    log_terrain_changed.in_set(ChunkSystemSet::ChunkReload),
                    clear_chunk_registry.in_set(ChunkSystemSet::ChunkReload),
                    despawn_entities::<Chunk>.in_set(ChunkSystemSet::ChunkReload),
                    spawn_chunks.in_set(ChunkSystemSet::Mutate),
                    render_chunks.in_set(ChunkSystemSet::Render),
                )
                    .chain()
                    .run_if(resource_changed::<TerrainSampler>),
            ),
        );
}

fn sync_terrain_with_ui(
    mut terrain: ResMut<TerrainSampler>,
    editor_state: Res<EditorTerrainState>,
) {
    terrain.noise_seed = editor_state.seed;
    terrain.noise_weights = editor_state
        .weights
        .iter()
        .filter(|(is_visible, _)| *is_visible)
        .map(|(_, w)| *w)
        .collect_vec();
}

fn log_terrain_changed() {
    info!("Terrain changed");
}

pub fn change_map_seed(mut terrain: ResMut<TerrainSampler>) {
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

fn editor_terrain_changed(editor_state: Res<EditorTerrainState>) -> bool {
    editor_state.manual_has_changed
}
fn unflag_manual_terrain_change(mut editor_state: ResMut<EditorTerrainState>) {
    editor_state.manual_has_changed = false;
}

fn editor_terrain_previews_changed(previews_state: Res<EditorTerrainImages>) -> bool {
    previews_state.manual_has_changed
}
fn unflag_manual_terrain_previews_change(mut previews_state: ResMut<EditorTerrainImages>) {
    previews_state.manual_has_changed = false;
}

impl Default for EditorTerrainState {
    fn default() -> Self {
        let terrain = TerrainSampler::default();

        Self {
            manual_has_changed: false,
            weights: terrain.noise_weights.iter().map(|w| (true, *w)).collect(),
            seed: terrain.noise_seed,
            chunk_subdivisions: terrain.chunk_subdivisions,
            chunk_spawn_radius: terrain.chunk_spawn_radius,
            chunk_visibility_radius: terrain.chunk_visibility_radius,
        }
    }
}

fn update_terrain_previews(
    ui_context: Query<&bevy_inspector_egui::bevy_egui::EguiContext, With<PrimaryWindow>>,
    mut terrain_images: ResMut<EditorTerrainImages>,
    editor_terrain: Res<EditorTerrainState>,
    render_center: Res<MapRenderCenter>,
) {
    let Ok(egui_context) = ui_context.get_single() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();
    let preview_scale = terrain_images.preview_scale;
    let offset = render_center.to_world_pos().xz();

    let highest_amplitude = editor_terrain
        .weights
        .iter()
        .map(|(_, w)| w)
        .max_by(|a, b| a.amplitude.partial_cmp(&b.amplitude).unwrap())
        .map(|w| w.amplitude)
        .unwrap_or(1.0);

    let hasher = noise::SimpleHasher::from_seed(editor_terrain.seed);
    let mut passed_weights = Vec::with_capacity(editor_terrain.weights.len());

    let mut weights_preview = Vec::with_capacity(editor_terrain.weights.len());
    for (is_visible, weight) in &editor_terrain.weights {
        if !*is_visible {
            continue;
        }
        passed_weights.push(weight);

        let individual = create_preview_texture(
            ctx,
            [DEBUG_IMAGE_SIZE, DEBUG_IMAGE_SIZE],
            preview_scale,
            |pos| {
                weight
                    .sample_erosion_base(&hasher, pos + offset, 0.0)
                    .0
                    .value
                    / highest_amplitude
            },
        );

        let combined = create_preview_texture(
            ctx,
            [DEBUG_IMAGE_SIZE, DEBUG_IMAGE_SIZE],
            preview_scale,
            |pos| {
                TerrainWeight::sample_many(
                    hasher.clone(),
                    pos + offset,
                    passed_weights.iter().map(|w| w as &_),
                )
                .0
                .value
                    / highest_amplitude
            },
        );

        let erosion_individual = create_preview_texture(
            ctx,
            [DEBUG_IMAGE_SIZE, DEBUG_IMAGE_SIZE],
            preview_scale,
            |pos| weight.sample_erosion_base(&hasher, pos + offset, 0.0).1,
        );

        let erosion_combined = create_preview_texture(
            ctx,
            [DEBUG_IMAGE_SIZE, DEBUG_IMAGE_SIZE],
            preview_scale,
            |pos| {
                TerrainWeight::sample_many(
                    hasher.clone(),
                    pos + offset,
                    passed_weights.iter().map(|w| w as &_),
                )
                .1
            },
        );

        weights_preview.push(TerrainPreview {
            individual,
            combined,

            erosion_individual,
            erosion_combined,
        });
    }

    terrain_images.weights_preview = weights_preview;
}

pub struct EditorTerrain {}

impl EditorDock for EditorTerrain {
    fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
        world.resource_scope::<EditorTerrainImages, _>(|world, mut terrain_images| {
            world.resource_scope::<EditorTerrainState, _>(|world, mut state| {
                // let is_focused = ui.memory(|m| m.clone()).focused().is_some();

                ui.collapsing("Preview", |ui| {
                    terrain_images.manual_has_changed = ui
                        .add(
                            egui::Slider::new(&mut terrain_images.preview_scale, 0.01..=1000.0)
                                .text("preview scale"),
                        )
                        .changed();

                    for weight_preview in terrain_images.weights_preview.iter() {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Individual");
                                ui.image(&weight_preview.individual);
                                // ui.image(&weight_preview.erosion_individual);
                            });
                            ui.vertical(|ui| {
                                ui.label("Combined");
                                ui.image(&weight_preview.combined);
                                // ui.image(&weight_preview.erosion_combined);
                            });
                        });
                        ui.separator();
                        ui.spacing();
                    }
                });

                ui.collapsing("Settings", |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Seed:");
                            if bevy_inspector::ui_for_value(&mut state.seed, ui, world) {
                                let mut terrain =
                                    world.get_resource_mut::<TerrainSampler>().unwrap();
                                terrain.noise_seed = state.seed;
                                state.manual_has_changed = true;
                            }
                        });
                        ui.vertical(|ui| {
                            ui.label("Mesh Subdivisions:");
                            if bevy_inspector::ui_for_value(
                                &mut state.chunk_subdivisions,
                                ui,
                                world,
                            ) {
                                let mut terrain =
                                    world.get_resource_mut::<TerrainSampler>().unwrap();
                                terrain.chunk_subdivisions = state.chunk_subdivisions;
                                state.manual_has_changed = true;
                            }
                        });
                        ui.vertical(|ui| {
                            ui.label("Visibility radius:");
                            if bevy_inspector::ui_for_value(
                                &mut state.chunk_visibility_radius,
                                ui,
                                world,
                            ) {
                                let mut terrain =
                                    world.get_resource_mut::<TerrainSampler>().unwrap();
                                terrain.chunk_visibility_radius = state.chunk_visibility_radius;
                                state.manual_has_changed = true;
                            }
                        });
                        ui.vertical(|ui| {
                            ui.label("Spawn radius:");
                            if bevy_inspector::ui_for_value(
                                &mut state.chunk_spawn_radius,
                                ui,
                                world,
                            ) {
                                let mut terrain =
                                    world.get_resource_mut::<TerrainSampler>().unwrap();
                                terrain.chunk_spawn_radius = state.chunk_spawn_radius;
                                state.manual_has_changed = true;
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Weights:");
                        if bevy_inspector::ui_for_value(&mut state.weights, ui, world) {
                            let mut terrain = world.get_resource_mut::<TerrainSampler>().unwrap();
                            terrain.noise_weights = state.weights.iter().map(|(_, w)| *w).collect();
                            state.manual_has_changed = true;
                        }
                    });
                });
            });
        });
    }
}

fn create_preview_texture(
    ctx: &egui::Context,
    size: [usize; 2],
    scale: f32,
    sample: impl Fn(Vec2) -> f32,
) -> egui::TextureHandle {
    let mut image = egui::ColorImage {
        size,
        pixels: vec![egui::Color32::BLACK; size[0] * size[1]],
    };
    let size_x_half = size[0] as i32 / 2;
    let size_y_half = size[1] as i32 / 2;
    for x in -size_x_half..size_x_half {
        for y in -size_y_half..size_y_half {
            let pos = vec2(x as f32 * scale, y as f32 * scale);
            let value = sample(pos) * 255.0;

            let color = egui::Color32::from_gray(value as u8);
            let x_pos = (x + size_x_half) as usize;
            let y_pos = (y + size_y_half) as usize;
            image.pixels[x_pos + y_pos * size[1]] = color;
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

fn display_world_grid(mut gizmos: Gizmos, query: Query<&Transform, With<ChunkOrigin>>) {
    for transform in query.iter() {
        let origin = transform.translation.floor();
        let range = 5;
        let color = tailwind::GRAY_400;
        for x in -range..range {
            gizmos.line(
                origin + vec3(x as f32, 0.0, -range as f32),
                origin + vec3(x as f32, 0.0, range as f32),
                color,
            );
        }
        for z in -range..range {
            gizmos.line(
                origin + vec3(-range as f32, 0.0, z as f32),
                origin + vec3(range as f32, 0.0, z as f32),
                color,
            );
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
