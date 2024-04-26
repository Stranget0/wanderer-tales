use std::fmt::Debug;

use bevy::{
    math::*,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};

use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

use bevy_easings::Lerp;
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
        .insert_resource(MapLOD::new())
        .add_systems(
            Update,
            (
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

const QUAD_TREE_DIRECTIONS: [Vec2; 4] = [
    Vec2::new(-1.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
];

#[derive(Debug, Clone, PartialEq)]
enum FixedTreeNode<const SIZE: usize, T> {
    Data(T),
    Split([Box<FixedTreeNode<SIZE, T>>; SIZE]),
}

impl<const SIZE: usize, T: Clone + Debug> FixedTreeNode<SIZE, Option<T>> {
    pub fn empty() -> Self {
        Self::Data(None)
    }
    pub fn empty_split() -> Self {
        let mut vec = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            vec.push(Box::new(Self::empty()));
        }
        Self::Split(vec.try_into().unwrap())
    }
}

trait TreeSetter {
    type TreeNode;

    fn handle_node(&self, parent: &mut Self::TreeNode, parent_center: Vec2, depth: usize);
}

impl<const SIZE: usize, T> FixedTreeNode<SIZE, T> {
    pub fn is_leaf(&self) -> bool {
        match self {
            FixedTreeNode::Data(_) => true,
            FixedTreeNode::Split(_) => false,
        }
    }

    pub fn iter(&self) -> FixedTreeNodeIterator<'_, SIZE, T> {
        FixedTreeNodeIterator { stack: vec![self] }
    }

    pub fn update_with_setter<S: TreeSetter<TreeNode = Self>>(&mut self, setter: &S) {
        setter.handle_node(self, Vec2::ZERO, 1);
    }
}

struct FixedTreeNodeIterator<'a, const SIZE: usize, T> {
    stack: Vec<&'a FixedTreeNode<SIZE, T>>,
}

impl<'a, const SIZE: usize, T> Iterator for FixedTreeNodeIterator<'a, SIZE, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = match self.stack.pop() {
                Some(n) => n,
                None => return None,
            };

            match node {
                FixedTreeNode::Data(ref data) => return Some(data),
                FixedTreeNode::Split(ref children) => {
                    for child in children.iter() {
                        self.stack.push(child);
                    }
                }
            }
        }
    }
}

type OcTree<T> = FixedTreeNode<8, T>;
type QuadTree<T> = FixedTreeNode<4, T>;

type MapChunkNode = QuadTree<Option<Entity>>;

#[derive(Debug, Resource)]
struct MapLOD(MapChunkNode);
impl MapLOD {
    pub fn new() -> Self {
        Self(MapChunkNode::empty())
    }
}

struct TreeCreatorMap {
    pub view_distance: u16,
    pub ref_distance: u16,
    pub min_precision: u16,
    pub ref_precision: u16,
}

impl TreeCreatorMap {
    pub fn distance_to_precision(&self, distance: f32) -> u16 {
        let factor = distance / self.ref_distance as f32;
        self.min_precision.lerp(&self.ref_precision, &factor)
    }
    pub fn should_subdivide(&self, current_precision: u16, distance: f32) -> bool {
        current_precision < self.distance_to_precision(distance)
    }

    pub fn get_size(&self, depth: usize) -> u16 {
        self.view_distance / depth as u16
    }
}

impl TreeSetter for TreeCreatorMap {
    type TreeNode = MapChunkNode;
    fn handle_node(&self, parent: &mut Self::TreeNode, parent_center: Vec2, parent_depth: usize) {
        match (
            parent.is_leaf(),
            self.should_subdivide(parent_depth as u16, parent_center.length()),
        ) {
            (true, true) => {
                *parent = MapChunkNode::empty_split();
            }
            (false, false) => {
                *parent = MapChunkNode::empty();
            }
            _ => (),
        }

        if let MapChunkNode::Split(children) = parent {
            let depth = parent_depth + 1;
            let size = self.get_size(depth);
            for (i, ch) in children.iter_mut().enumerate() {
                let direction = QUAD_TREE_DIRECTIONS[i];
                let new_center = parent_center + direction * size as f32;

                self.handle_node(ch, new_center, depth)
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use crate::{FixedTreeNode, MapLOD, TreeCreatorMap};

    #[test]
    fn tree_traverse() {
        let tree: FixedTreeNode<4, i32> = FixedTreeNode::Split([
            Box::new(FixedTreeNode::Data(0)),
            Box::new(FixedTreeNode::Data(1)),
            Box::new(FixedTreeNode::Data(2)),
            Box::new(FixedTreeNode::Split([
                Box::new(FixedTreeNode::Data(3)),
                Box::new(FixedTreeNode::Data(4)),
                Box::new(FixedTreeNode::Data(5)),
                Box::new(FixedTreeNode::Data(6)),
            ])),
        ]);
        for (i, node) in tree.iter().enumerate() {}
    }

    #[test]
    fn map_lod_create() {
        let mut tree = MapLOD::new();
        let setter = TreeCreatorMap {
            view_distance: 100,
            ref_distance: 50,
            min_precision: 2,
            ref_precision: 5,
        };
        tree.0.update_with_setter(&setter);

        assert!(!tree.0.is_leaf(), "top node shouldnt be leaf");
    }
}
