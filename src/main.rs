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
                systems::track_chunks_to_spawn,
                systems::track_chunks_to_despawn,
                systems::spawn_chunks.after(systems::track_chunks_to_spawn),
                systems::despawn_chunks.after(systems::track_chunks_to_despawn),
                systems::render_chunks,
            ),
        )
        .run();
}

const CHUNK_SUBDIVISIONS: usize = 4;
const CHUNK_ITEM_COUNT: usize = CHUNK_SUBDIVISIONS * CHUNK_SUBDIVISIONS;
const CHUNK_SIZE: f32 = 10.0;

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

#[derive(Debug)]
struct Height(u8);

impl Height {
    pub fn to_world_y(&self) -> f32 {
        self.0 as f32 / 5.0
    }
}

impl IChunkPos {
    pub fn to_world_pos(self) -> Vec3 {
        Vec3::new(self.0 as f32 * CHUNK_SIZE, 0.0, self.1 as f32 * CHUNK_SIZE)
    }
}

impl IChunkPos {
    pub fn as_vec(&self) -> IVec2 {
        IVec2::new(self.0, self.1)
    }
}

impl HeightChunk {
    pub fn get_index(x: usize, y: usize) -> usize {
        x + y * CHUNK_SUBDIVISIONS
    }
    pub fn get_height(&self, x: usize, y: usize) -> Height {
        Height(self.0[Self::get_index(x, y)])
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
    use bevy::render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    };

    use super::*;

    pub fn track_chunks_to_spawn(
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
                if IChunkPos(x, y).to_world_pos().distance(player_pos)
                    > chunk_view_distance.0.into()
                {
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
        let player_pos = player_query.single().translation;

        for (chunk_entity, chunk_pos) in existing_chunks.iter() {
            if player_pos.distance(chunk_pos.to_world_pos()) > chunk_view_distance.0 as f32 {
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
                        mesh: asset_server.add(create_subdivided_plane(CHUNK_SIZE, &chunk_pos)),
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
        for x in 0..CHUNK_SUBDIVISIONS {
            for y in 0..CHUNK_SUBDIVISIONS {
                // 0.0 - 1.0
                let pos = IChunkPos(x as i32, y as i32).to_world_pos() + pos.to_world_pos();
                let height_f = simplex_noise_2d_seeded(vec2(pos.x, pos.z), 0.0) / 2.0 + 0.5;

                // 0..max
                let height = (max_height.as_f32() * height_f) as u8;

                chunk_heights.set_height(x, y, height);
                chunk_heights.set_height(x, y, height);
                chunk_heights.set_height(x, y, height);
            }
        }

        chunk_heights
    }

    fn create_subdivided_plane(size: f32, chunk_pos: &IChunkPos) -> Mesh {
        let subdivisions: u16 = CHUNK_SUBDIVISIONS as u16 - 1;
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        // Calculate the step size for each subdivision
        let step = size / subdivisions as f32;
        let pos = chunk_pos.to_world_pos();
        // Generate vertices, UVs, and normals
        for i in 0..=subdivisions {
            for j in 0..=subdivisions {
                let x = i as f32 * step - 0.5;
                // let y = heights.get_height(j.into(), i.into()).to_world_y();
                let z = j as f32 * step - 0.5;
                let y = simplex_noise_2d_seeded(vec2(pos.x + x, pos.z + z), 0.0);
                info!("{} {} {}", x, y, z);
                vertices.push([x, y, z]);
                uvs.push([x + 0.5, z + 0.5]);
                normals.push([0.0, 1.0, 0.0]); // All normals point straight up
            }
        }

        // Generate indices for triangles
        for i in 0..subdivisions {
            for j in 0..subdivisions {
                let index = i * (subdivisions + 1) + j;

                indices.push(index);
                indices.push(index + 1);
                indices.push(index + subdivisions + 1);

                indices.push(index + 1);
                indices.push(index + subdivisions + 2);
                indices.push(index + subdivisions + 1);
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
}
