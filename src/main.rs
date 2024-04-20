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
        .configure_sets(Update, MapSet::Data.before(MapSet::Render))
        .add_systems(
            Update,
            (
                (
                    systems::generate_height_chunk,
                    systems::spawn_chunks,
                    systems::despawn_chunks,
                )
                    .in_set(MapSet::Data),
                (systems::render_chunks, systems::despawn_render_chunks).in_set(MapSet::Render),
            ),
        )
        .run();
}

const CHUNK_SIZE: usize = 32;
const CHUNK_ITEM_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Component)]
struct MapChunk;

#[derive(Debug, Component)]
struct HeightChunk([u8; CHUNK_ITEM_COUNT]);
#[derive(Debug, Component)]
struct AwaitingHeightChunk;

#[derive(Debug, Component)]
struct AwaitingRenderChunk;

#[derive(Debug, Component)]
struct RenderChunkLink(pub Entity);

#[derive(Debug, Component, PartialEq, Eq)]
struct IWorldPos(pub i32, pub i32);

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

impl Into<AssetId<Mesh>> for MeshType {
    fn into(self) -> AssetId<Mesh> {
        match self {
            MeshType::Plane => AssetId::Uuid {
                uuid: Uuid::parse_str("06e3459d-9243-4cba-abf7-61d43651748e").unwrap(),
            },
        }
    }
}

impl Into<AssetId<StandardMaterial>> for MaterialType {
    fn into(self) -> AssetId<StandardMaterial> {
        match self {
            MaterialType::Debug => AssetId::Uuid {
                uuid: Uuid::parse_str("bae74ba4-6b88-4198-806a-a956d1ac7d9d").unwrap(),
            },
        }
    }
}

impl IWorldPos {
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

    pub fn spawn_chunks(
        mut commands: Commands,
        player_query: Query<&Transform, With<FlyCam>>,
        existing_chunks: Query<&IWorldPos, With<MapChunk>>,
        chunk_view_distance: Res<ChunkViewDistance>,
    ) {
        let player_pos = {
            let pos = player_query.single().translation;
            ivec2(pos.x as i32, pos.y as i32)
        };

        let mut chunks = existing_chunks.iter().collect_vec();
        let mut bundles = Vec::<(MapChunk, IWorldPos, Name, AwaitingHeightChunk)>::with_capacity(
            (chunk_view_distance.0 * chunk_view_distance.0).into(),
        );

        let from = player_pos.to_array();
        let to = [
            from[0] + chunk_view_distance.0 as i32,
            from[1] + chunk_view_distance.0 as i32,
        ];

        for x in from[0]..to[0] {
            for y in from[1]..to[1] {
                let pos = IWorldPos(x, y);
                let chunk_option = chunks.iter().position(|&chunk_pos| &pos == chunk_pos);
                if let Some(chunk_index) = chunk_option {
                    chunks.swap_remove(chunk_index);
                } else {
                    let bundle = (
                        MapChunk,
                        IWorldPos(x, y),
                        Name::new("Chunk"),
                        AwaitingHeightChunk,
                    );
                    bundles.push(bundle);
                }
            }
        }
        commands.spawn_batch(bundles);
    }

    pub fn despawn_chunks(
        mut commands: Commands,
        player_query: Query<&Transform, With<FlyCam>>,
        existing_chunks: Query<(Entity, &IWorldPos), With<MapChunk>>,
        chunk_view_distance: Res<ChunkViewDistance>,
    ) {
        let player_pos = {
            let pos = player_query.single().translation;
            ivec2(pos.x as i32, pos.y as i32)
        };

        for (chunk_entity, chunk_pos) in existing_chunks.iter() {
            if player_pos.distance_squared(chunk_pos.as_vec()) > chunk_view_distance.0 as i32 {
                commands.entity(chunk_entity).despawn_recursive();
            }
        }
    }

    pub fn generate_height_chunk(
        mut commands: Commands,
        chunks_query: Query<Entity, With<AwaitingHeightChunk>>,
        max_height: Res<MaxHeight>,
    ) {
        for chunk in chunks_query.iter() {
            let mut chunk_heights = HeightChunk::default();
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    // 0.0 - 1.0
                    let height_f =
                        simplex_noise_2d_seeded(vec2(x as f32, y as f32), 0.0) / 2.0 + 0.5;

                    // 0..max
                    let height = (max_height.as_f32() * height_f) as u8;

                    chunk_heights.set_height(x, y, height);
                }
            }
            commands
                .entity(chunk)
                .insert(chunk_heights)
                .insert(AwaitingRenderChunk)
                .remove::<AwaitingHeightChunk>();
        }
    }

    pub fn render_chunks(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        chunks: Query<(Entity, &HeightChunk, &IWorldPos), With<AwaitingRenderChunk>>,
    ) {
        // let mesh_provider = meshes.get_handle_provider();
        let mesh = asset_server
            .get_id_handle(MeshType::Plane.into())
            .unwrap_or_else(|| asset_server.add(Plane3d::new(*Direction3d::Y).into()));

        let material = asset_server
            .get_id_handle(MaterialType::Debug.into())
            .unwrap_or_else(|| asset_server.add(Color::DARK_GREEN.into()));

        for (chunk, chunk_heights, world_pos) in chunks.iter() {
            let transform = Transform::from_xyz(world_pos.0 as f32, 0.0, world_pos.1 as f32);
            let render_id = commands
                .spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform,
                    ..default()
                })
                .id();

            commands
                .entity(chunk)
                .insert(RenderChunkLink(render_id))
                .remove::<AwaitingRenderChunk>();
        }
    }

    pub fn despawn_render_chunks(
        mut commands: Commands,
        mut removals: RemovedComponents<RenderChunkLink>,
        render_link: Query<&RenderChunkLink>,
    ) {
        for entity in removals.read() {
            match render_link.get(entity) {
                Ok(render_link) => commands.entity(render_link.0).despawn_recursive(),
                Err(err) => error!("Could not remove render item {}", err),
            }
        }
    }
}
