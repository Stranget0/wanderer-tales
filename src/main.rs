use bevy::{
    math::*,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_editor_pls::EditorPlugin;
use bevy_flycam::{FlyCam, PlayerPlugin};
use itertools::Itertools;
use noisy_bevy::{simplex_noise_2d_seeded, NoisyShaderPlugin};
use wanderer_tales::debug::fps_counter::FPSPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            NoisyShaderPlugin,
            EditorPlugin::default(),
            PlayerPlugin,
            FPSPlugin,
        ))
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::YELLOW_GREEN,
        })
        .insert_resource(ChunkViewDistance(100))
        .insert_resource(MaxHeight(3))
        .add_event::<SpawnChunkEvent>()
        .add_event::<DespawnChunkEvent>()
        .add_systems(
            Update,
            (
                track_chunks_to_spawn,
                track_chunks_to_despawn,
                spawn_chunks.after(track_chunks_to_spawn),
                despawn_chunks.after(track_chunks_to_despawn),
                render_chunks,
            ),
        )
        .run();
}

const CHUNK_SLICES: usize = 4;
const CHUNK_ITEM_COUNT: usize = CHUNK_SLICES * CHUNK_SLICES;
const CHUNK_SIZE: f32 = 10.0;
const CHUNK_SIZE_HALF: f32 = CHUNK_SIZE / 2.0;

#[derive(Debug, Event)]
struct SpawnChunkEvent {
    pub pos: IChunkPos,
}
#[derive(Debug, Event)]
struct DespawnChunkEvent(pub Entity);

#[derive(Debug, Component)]
struct MapChunk;

#[derive(Debug, Component)]
struct ChunkHeights([u8; CHUNK_ITEM_COUNT]);

#[derive(Debug, Component)]
struct RenderChunkLink(pub Vec<Entity>);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
struct IChunkPos(pub i32, pub i32);
#[derive(Debug, Clone, Copy, Component, PartialEq)]
struct FChunkPos(pub f32, pub f32);

#[derive(Debug, Resource)]
struct ChunkViewDistance(u16);

#[derive(Debug, Resource)]
struct MaxHeight(u16);

#[derive(Debug)]
enum MeshType {
    Plane,
}

#[derive(Debug)]
enum MaterialType {
    Debug,
}

#[derive(Debug)]
struct Height(u8);

impl Height {
    pub fn to_world_y(&self) -> f32 {
        self.0 as f32 / 5.0
    }
}

impl IChunkPos {
    pub fn to_world_pos(self) -> Vec3 {
        Vec3::new(
            self.0 as f32 * CHUNK_SIZE - CHUNK_SIZE_HALF,
            0.0,
            self.1 as f32 * CHUNK_SIZE - CHUNK_SIZE_HALF,
        )
    }
}
impl FChunkPos {
    pub fn to_world_pos(self) -> Vec3 {
        Vec3::new(
            self.0 * CHUNK_SIZE - CHUNK_SIZE_HALF,
            0.0,
            self.1 * CHUNK_SIZE - CHUNK_SIZE_HALF,
        )
    }
}

impl From<IChunkPos> for FChunkPos {
    fn from(value: IChunkPos) -> Self {
        Self(value.0 as f32, value.1 as f32)
    }
}

impl ChunkHeights {
    pub fn get_index(x: usize, y: usize) -> usize {
        x + y * CHUNK_SLICES
    }
    pub fn get_height(&self, x: usize, y: usize) -> Height {
        Height(self.0[Self::get_index(x, y)])
    }
    pub fn set_height(&mut self, x: usize, y: usize, height: u8) {
        self.0[Self::get_index(x, y)] = height;
    }
}

impl Default for ChunkHeights {
    fn default() -> Self {
        Self([0; CHUNK_ITEM_COUNT])
    }
}

impl MaxHeight {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

fn track_chunks_to_spawn(
    player_query: Query<&Transform, With<FlyCam>>,
    existing_chunks: Query<&IChunkPos, With<MapChunk>>,
    chunk_view_distance: Res<ChunkViewDistance>,
    mut spawn_chunk: EventWriter<SpawnChunkEvent>,
) {
    let player_pos = player_query.single().translation;
    let player_pos_2d = IVec2::new(player_pos.x as i32, player_pos.z as i32);

    let mut chunks = existing_chunks.iter().collect_vec();
    let mut events = Vec::<SpawnChunkEvent>::with_capacity(
        (chunk_view_distance.0 * chunk_view_distance.0).into(),
    );

    let radius = IVec2::splat(chunk_view_distance.0.into());
    let from = (player_pos_2d - radius).to_array();
    let to = (player_pos_2d + radius).to_array();

    for x in from[0]..to[0] {
        for y in from[1]..to[1] {
            let chunk_pos = IChunkPos(x, y).to_world_pos();
            let distance = chunk_pos.distance(player_pos);

            if distance > chunk_view_distance.0.into() {
                continue;
            }
            let pos = IChunkPos(x, y);
            let chunk_option = chunks.iter().position(|&chunk_pos| &pos == chunk_pos);
            if let Some(chunk_index) = chunk_option {
                info!("{} - {} => {}", chunk_pos, player_pos, distance);
                chunks.swap_remove(chunk_index);
            } else {
                let pos = IChunkPos(x, y);
                events.push(SpawnChunkEvent { pos })
            }
        }
    }
    spawn_chunk.send_batch(events);
}

fn track_chunks_to_despawn(
    player_query: Query<&Transform, With<FlyCam>>,
    existing_chunks: Query<(Entity, &IChunkPos), With<MapChunk>>,
    chunk_view_distance: Res<ChunkViewDistance>,
    mut despawn_chunks: EventWriter<DespawnChunkEvent>,
) {
    let mut events = Vec::with_capacity(chunk_view_distance.0 as usize * 4);
    let player_pos = player_query.single().translation;

    for (chunk_entity, chunk_pos) in existing_chunks.iter() {
        if player_pos.distance(chunk_pos.to_world_pos()) > chunk_view_distance.0 as f32 {
            events.push(DespawnChunkEvent(chunk_entity))
        }
    }
    despawn_chunks.send_batch(events);
}

fn spawn_chunks(
    mut commands: Commands,
    max_height: Res<MaxHeight>,
    mut spawn_chunk: EventReader<SpawnChunkEvent>,
) {
    let chunks = spawn_chunk
        .read()
        .map(|SpawnChunkEvent { pos }| {
            (
                MapChunk,
                generate_height_chunk(&max_height, pos),
                *pos,
                Name::new(format!("Chunk_{}x{}", pos.0, pos.1)),
                RenderChunkLink(Vec::with_capacity(2)),
            )
        })
        .collect_vec();
    if !chunks.is_empty() {
        info!("Spawning {} chunks", chunks.len());
        commands.spawn_batch(chunks);
    }
}

fn render_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut chunks: Query<
        (Entity, &ChunkHeights, &IChunkPos, &mut RenderChunkLink),
        Or<(Added<ChunkHeights>, Changed<ChunkHeights>)>,
    >,
) {
    // let mesh = asset_server
    //     .get_id_handle(MeshType::Plane.into())
    //     .unwrap_or_else(|| asset_server.add(Plane3d::new(*Direction3d::Y).into()));

    let material = asset_server.add(Color::DARK_GREEN.into());

    for (chunk, chunk_heights, chunk_pos, mut links) in chunks.iter_mut() {
        let transform = Transform::from_translation(chunk_pos.to_world_pos());

        let render_id = commands
            .spawn((
                Name::new("RenderChunk"),
                PbrBundle {
                    mesh: asset_server.add(create_subdivided_plane(CHUNK_SIZE, chunk_heights)),
                    material: material.clone(),
                    transform,
                    ..default()
                },
            ))
            .id();

        links.0.push(render_id);
    }
}

fn despawn_chunks(
    mut commands: Commands,
    mut despawn_chunks: EventReader<DespawnChunkEvent>,
    links_query: Query<&RenderChunkLink>,
) {
    if !despawn_chunks.is_empty() {
        info!("Despawning {} chunks", despawn_chunks.len());
    }
    for e in despawn_chunks.read() {
        if let Ok(links) = links_query.get(e.0) {
            for e in &links.0 {
                commands.entity(*e).despawn_recursive();
            }
        }
        commands.entity(e.0).despawn_recursive();
    }
}

fn generate_height_chunk(max_height: &Res<MaxHeight>, pos: &IChunkPos) -> ChunkHeights {
    let mut chunk_heights = ChunkHeights::default();
    let max_index = CHUNK_SLICES as f32 - 1.0;
    for z in 0..CHUNK_SLICES {
        for x in 0..CHUNK_SLICES {
            // 0.0 - 1.0
            let sub_chunk_pos = FChunkPos(x as f32 / max_index, z as f32 / max_index);

            let chunk_world_pos = pos.to_world_pos() + sub_chunk_pos.to_world_pos();

            let height_f = simplex_noise_2d_seeded(vec2(chunk_world_pos.x, chunk_world_pos.z), 0.0)
                / 2.0
                + 0.5;

            // 0..max
            let height = (max_height.as_f32() * height_f) as u8;

            chunk_heights.set_height(x, z, height);
        }
    }

    chunk_heights
}

fn create_subdivided_plane(size: f32, chunk_heights: &ChunkHeights) -> Mesh {
    let slices = CHUNK_SLICES;
    let total = (slices - 1) as f32;
    let capacity = slices * 6;
    let mut vertices = Vec::with_capacity(capacity);
    let mut uvs = Vec::with_capacity(capacity);
    let mut normals = Vec::with_capacity(capacity);
    let mut indices = Vec::with_capacity(capacity);

    for i in 0..slices {
        for j in 0..slices {
            let x = j as f32 / total;
            let z = i as f32 / total;

            let mut pos = FChunkPos(x, z).to_world_pos();
            pos.y = chunk_heights.get_height(j, i).to_world_y();

            vertices.push(pos);
            uvs.push([x, z]);
            normals.push([0.0, 1.0, 0.0]);
        }
    }

    let total_u = total as u16;
    for i in 0..total_u {
        for j in 0..total_u {
            let index_1 = j + i * (total_u + 1);
            let index_2 = index_1 + 1;
            let index_3 = index_1 + total_u + 2;
            let index_4 = index_1 + total_u + 1;

            // Triangle 1
            indices.push(index_3);
            indices.push(index_2);
            indices.push(index_1);

            // Triangle 2
            indices.push(index_4);
            indices.push(index_3);
            indices.push(index_1);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U16(indices))
}
