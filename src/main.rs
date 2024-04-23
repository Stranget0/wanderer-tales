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
        .insert_resource(MapChunkOctree(FixedTreeNode::Data(None)))
        .add_event::<SpawnChunkEvent>()
        .add_event::<DespawnChunkEvent>()
        .add_systems(
            Update,
            (
                track_map_quadtree,
                spawn_quadtree_chunks,
                render_chunks,
                // track_chunks_to_spawn,
                // track_chunks_to_despawn,
                // spawn_chunks.after(track_chunks_to_spawn),
                // despawn_chunks.after(track_chunks_to_despawn),
                // render_chunks,
            ),
        )
        .run();
}

const CHUNK_SLICES: usize = 4;
const CHUNK_ITEM_COUNT: usize = CHUNK_SLICES * CHUNK_SLICES;
const CHUNK_SIZE: f32 = 1000.0;
const CHUNK_SIZE_HALF: f32 = CHUNK_SIZE / 2.0;

#[derive(Debug, Clone)]
enum FixedTreeNode<const i: usize, T> {
    Data(T),
    Split([Box<FixedTreeNode<i, T>>; i]),
}

type OcTree<T> = FixedTreeNode<8, T>;
type QuadTree<T> = FixedTreeNode<4, T>;

type MapChunkNode = QuadTree<Option<Entity>>;
#[derive(Resource)]
struct MapChunkOctree(MapChunkNode);

impl MapChunkOctree {
    pub fn get_data_from_node(node: &MapChunkNode) -> Option<&Entity> {
        match node {
            FixedTreeNode::Data(Some(entity)) => Some(entity),
            _ => None,
        }
    }

    pub fn get_data_from_child(parent: &MapChunkNode, i: usize) -> Option<&Entity> {
        match parent {
            FixedTreeNode::Split(nodes) => Self::get_data_from_node(&nodes[i]),
            _ => None,
        }
    }

    pub fn get_child(parent: &MapChunkNode, i: usize) -> Option<&MapChunkNode> {
        match parent {
            FixedTreeNode::Split(arr) => Some(&arr[i]),
            _ => None,
        }
    }
    pub fn get_child_mut(parent: &mut MapChunkNode, i: usize) -> Option<&mut MapChunkNode> {
        match parent {
            FixedTreeNode::Split(arr) => Some(&mut arr[i]),
            _ => None,
        }
    }
}
#[derive(Debug, Event)]
struct SpawnOctreeChunkEvent {
    tree_path: Vec<usize>,
    center_pos: Vec2,
    size: f32,
    slices: usize,
}

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
struct ChunkHeightsVec(Vec<u8>);

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
impl ChunkHeightsVec {
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

fn track_map_quadtree(
    player_query: Query<&Transform, With<FlyCam>>,
    chunk_view_distance: Res<ChunkViewDistance>,
    mut octree: ResMut<MapChunkOctree>,
    mut spawn_chunk: EventWriter<SpawnOctreeChunkEvent>,

    last_request_location: Local<Option<Vec3>>,
) {
    let player_location = player_query.single().translation;

    if last_request_location
        .is_some_and(|last_location| player_location.distance(last_location) < 5.0)
    {
        return;
    };

    // let path = Vec::with_capacity(16);
    // let mut events = Vec::new();

    update_tree(
        &mut octree.0,
        Vec2::ZERO,
        0,
        chunk_view_distance.0.into(),
        chunk_view_distance.0.into(),
    );

    spawn_chunk.send_batch(events);
}

const QUAD_TREE_DIRECTIONS: [Vec2; 4] = [
    Vec2::new(-1.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
];

fn spawn_quadtree_chunks(
    mut commands: Commands,
    mut spawn_chunk: EventReader<SpawnOctreeChunkEvent>,
    chunk_view_distance: Res<ChunkViewDistance>,
    max_height: Res<MaxHeight>,
) {
    let mut chunks_to_spawn = Vec::with_capacity(spawn_chunk.len());
    for e in spawn_chunk.read() {
        if e.center_pos.length() > chunk_view_distance.0.into() {
            continue;
        }

        chunks_to_spawn.push((
            MapChunk,
            generate_height_chunk(&max_height, &e.center_pos, e.size),
            Name::new(format!("Chunk_{}x{}", &e.center_pos.x, e.center_pos.y)),
            RenderChunkLink(Vec::with_capacity(2)),
        ))
    }

    if !chunks_to_spawn.is_empty() {
        info!("Spawning {} chunks", chunks_to_spawn.len());
        commands.spawn_batch(chunks_to_spawn);
    }
}

fn update_tree(
    mut parent_node: &mut MapChunkNode,
    center_pos: Vec2,
    depth: i16,
    size: f32,
    // slices: usize,
    view_distance: f32,
    tree_path: &Vec<usize>,
    on_update_leaf: fn(center_pos: Vec2, node_path: &Vec<usize>, size: f32), // events_to_send: &mut Vec<SpawnOctreeChunkEvent>,
) {
    if let MapChunkNode::Data(_) = parent_node {
        let arr = (0..4)
            .map(|_| Box::new(MapChunkNode::Data(None)))
            .collect_vec()
            .try_into()
            .unwrap();

        *parent_node = MapChunkNode::Split(arr);
    }

    for i in 0..4 {
        let new_depth = depth + 1;
        let new_size = size / 2.0;
        let new_center_pos = center_pos + QUAD_TREE_DIRECTIONS[i] / 2.0 * new_size;
        let distance = new_center_pos.length();
        let mut node_path = tree_path.clone();
        node_path.push(i);
        let node = MapChunkOctree::get_child_mut(parent_node, i).unwrap();

        if distance < view_distance / new_depth as f32 && new_depth < 32 {
            update_tree(
                node,
                new_center_pos,
                new_depth,
                new_size,
                // slices,
                view_distance,
                &node_path,
                on_update_leaf, // events_to_send,
            )
        } else if let MapChunkNode::Split(_) = node {
            *node = MapChunkNode::Data(None);

            // events_to_send.push(SpawnOctreeChunkEvent {
            //     center_pos: new_center_pos,
            //     tree_path: node_path,
            //     size: new_size,
            //     slices,
            // })
        }
    }
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
                    mesh: asset_server.add(create_subdivided_plane(
                        CHUNK_SIZE,
                        chunk_heights,
                        CHUNK_SLICES,
                    )),
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

fn generate_height_chunk(max_height: &Res<MaxHeight>, pos: &Vec2, size: f32) -> ChunkHeights {
    let mut chunk_heights = ChunkHeights::default();
    let max_index = CHUNK_SLICES as f32 - 1.0;
    for z in 0..CHUNK_SLICES {
        for x in 0..CHUNK_SLICES {
            // 0.0 - 1.0
            let sub_chunk_pos = FChunkPos(x as f32 / max_index, z as f32 / max_index);

            let height_f = simplex_noise_2d_seeded(vec2(pos.x, pos.y), 0.0) / 2.0 + 0.5;

            // 0..max
            let height = (max_height.as_f32() * height_f) as u8;

            chunk_heights.set_height(x, z, height);
        }
    }

    chunk_heights
}

fn create_subdivided_plane(size: f32, chunk_heights: &ChunkHeights, slices: usize) -> Mesh {
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
