use bevy::{
    math::*,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    utils::Uuid,
};
use bevy_editor_pls::EditorPlugin;
use bevy_flycam::{FlyCam, PlayerPlugin};
use itertools::Itertools;
use noisy_bevy::{simplex_noise_2d_seeded, NoisyShaderPlugin};
use wanderer_tales::debug::fps_counter::FPSPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum MapSet {
    UpdateStart,
    Data,
    Render,
}

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
            global: false,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::YELLOW_GREEN,
        })
        .insert_resource(ChunkViewDistance(3))
        .insert_resource(MaxHeight(255))
        .add_event::<SpawnChunkEvent>()
        .add_event::<DespawnChunkEvent>()
        .add_systems(
            Update,
            (
                systems::track_chunks_to_spawn,
                systems::track_chunks_to_despawn,
                systems::spawn_chunks.after(systems::track_chunks_to_spawn),
                systems::despawn_chunks.after(systems::track_chunks_to_despawn),
                systems::render_chunks,
            ),
        )
        .run();
}

const CHUNK_SIZE: usize = 32;
const CHUNK_ITEM_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Event)]
struct SpawnChunkEvent {
    pub pos: IChunkPos,
}
#[derive(Debug, Event)]
struct DespawnChunkEvent(pub Entity);

#[derive(Debug, Component)]
struct MapChunk;

#[derive(Debug, Component)]
struct HeightChunk([u8; CHUNK_ITEM_COUNT]);

#[derive(Debug, Component)]
struct RenderChunkLink(pub Vec<Entity>);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
struct IChunkPos(pub i32, pub i32);

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

impl From<MeshType> for AssetId<Mesh> {
    fn from(val: MeshType) -> Self {
        match val {
            MeshType::Plane => AssetId::Uuid {
                uuid: Uuid::parse_str("06e3459d-9243-4cba-abf7-61d43651748e").unwrap(),
            },
        }
    }
}

impl From<MaterialType> for AssetId<StandardMaterial> {
    fn from(val: MaterialType) -> Self {
        match val {
            MaterialType::Debug => AssetId::Uuid {
                uuid: Uuid::parse_str("bae74ba4-6b88-4198-806a-a956d1ac7d9d").unwrap(),
            },
        }
    }
}

impl IChunkPos {
    pub fn as_vec(&self) -> IVec2 {
        IVec2::new(self.0, self.1)
    }
}

impl HeightChunk {
    pub fn get_index(x: usize, y: usize) -> usize {
        x + y * CHUNK_SIZE
    }
    pub fn get_height(&self, x: usize, y: usize) -> u8 {
        self.0[Self::get_index(x, y)]
    }
    pub fn set_height(&mut self, x: usize, y: usize, height: u8) {
        self.0[Self::get_index(x, y)] = height;
    }
}

impl Default for HeightChunk {
    fn default() -> Self {
        Self([0; CHUNK_ITEM_COUNT])
    }
}

impl MaxHeight {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

mod systems {
    use super::*;

    pub fn track_chunks_to_spawn(
        player_query: Query<&Transform, With<FlyCam>>,
        existing_chunks: Query<&IChunkPos, With<MapChunk>>,
        chunk_view_distance: Res<ChunkViewDistance>,
        mut spawn_chunk: EventWriter<SpawnChunkEvent>,
    ) {
        let player_pos = {
            let pos = player_query.single().translation;
            ivec2(pos.x as i32, pos.z as i32)
        };

        let mut chunks = existing_chunks.iter().collect_vec();
        let mut events = Vec::<SpawnChunkEvent>::with_capacity(
            (chunk_view_distance.0 * chunk_view_distance.0).into(),
        );

        let from = player_pos.to_array();
        let to = [
            from[0] + chunk_view_distance.0 as i32,
            from[1] + chunk_view_distance.0 as i32,
        ];

        for x in from[0]..to[0] {
            for y in from[1]..to[1] {
                if ivec2(x, y).distance_squared(player_pos) > chunk_view_distance.0.into() {
                    continue;
                }
                let pos = IChunkPos(x, y);
                let chunk_option = chunks.iter().position(|&chunk_pos| &pos == chunk_pos);
                if let Some(chunk_index) = chunk_option {
                    chunks.swap_remove(chunk_index);
                } else {
                    let pos = IChunkPos(x, y);
                    events.push(SpawnChunkEvent { pos })
                }
            }
        }
        spawn_chunk.send_batch(events);
    }

    pub fn track_chunks_to_despawn(
        player_query: Query<&Transform, With<FlyCam>>,
        existing_chunks: Query<(Entity, &IChunkPos), With<MapChunk>>,
        chunk_view_distance: Res<ChunkViewDistance>,
        mut despawn_chunks: EventWriter<DespawnChunkEvent>,
    ) {
        let mut events = Vec::with_capacity(chunk_view_distance.0 as usize * 4);
        let player_pos = {
            let pos = player_query.single().translation;
            ivec2(pos.x as i32, pos.z as i32)
        };

        for (chunk_entity, chunk_pos) in existing_chunks.iter() {
            if player_pos.distance_squared(chunk_pos.as_vec()) > chunk_view_distance.0 as i32 {
                events.push(DespawnChunkEvent(chunk_entity))
            }
        }
        despawn_chunks.send_batch(events);
    }

    pub fn spawn_chunks(
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
                    Name::new("Chunk"),
                    RenderChunkLink(Vec::with_capacity(2)),
                )
            })
            .collect_vec();
        if !chunks.is_empty() {
            info!("Spawning {} chunks", chunks.len());
            commands.spawn_batch(chunks);
        }
    }

    pub fn render_chunks(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut chunks: Query<
            (Entity, &HeightChunk, &IChunkPos, &mut RenderChunkLink),
            Or<(Added<HeightChunk>, Changed<HeightChunk>)>,
        >,
    ) {
        let mesh = asset_server
            .get_id_handle(MeshType::Plane.into())
            .unwrap_or_else(|| asset_server.add(Plane3d::new(*Direction3d::Y).into()));

        let material = asset_server
            .get_id_handle(MaterialType::Debug.into())
            .unwrap_or_else(|| asset_server.add(Color::DARK_GREEN.into()));

        for (chunk, chunk_heights, world_pos, mut links) in chunks.iter_mut() {
            let transform = Transform::from_xyz(world_pos.0 as f32, 0.0, world_pos.1 as f32);
            let render_id = commands
                .spawn((
                    Name::new("RenderChunk"),
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform,
                        ..default()
                    },
                ))
                .id();

            links.0.push(render_id);
        }
    }

    pub fn despawn_chunks(
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

    fn generate_height_chunk(max_height: &Res<MaxHeight>, pos: &IChunkPos) -> HeightChunk {
        let mut chunk_heights = HeightChunk::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                // 0.0 - 1.0
                let height_f = simplex_noise_2d_seeded(
                    vec2(x as f32 + pos.0 as f32, y as f32 + pos.1 as f32),
                    0.0,
                ) / 2.0
                    + 0.5;

                // 0..max
                let height = (max_height.as_f32() * height_f) as u8;

                chunk_heights.set_height(x, y, height);
                chunk_heights.set_height(x, y, height);
                chunk_heights.set_height(x, y, height);
            }
        }

        chunk_heights
    }
}
