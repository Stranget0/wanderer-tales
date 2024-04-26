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
        .insert_resource(ChunkViewDistance(5))
        .insert_resource(MaxHeight(3))
        .insert_resource(MapLOD::new())
        .add_systems(Startup, (render_chunks))
        .add_systems(Update, (draw_gizmos))
        .run();
}

const CHUNK_SLICES: usize = 4;
const CHUNK_ITEM_COUNT: usize = CHUNK_SLICES * CHUNK_SLICES;

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

trait TreeSetter {
    type TreeNode;

    fn handle_node(&self, root: &mut Self::TreeNode);
}

impl<const SIZE: usize, T> FixedTreeNode<SIZE, T> {
    pub fn iter(&self) -> FixedTreeNodeIterator<'_, SIZE, T> {
        FixedTreeNodeIterator { stack: vec![self] }
    }

    pub fn update_with_setter<S: TreeSetter<TreeNode = Self>>(&mut self, setter: &S) {
        setter.handle_node(self);
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

#[derive(Debug, Clone)]
struct MapChunkDataNode {
    entity: Option<Entity>,
    pos: Vec2,
    size: f32,
    depth: usize,
}

impl MapChunkDataNode {
    pub fn new(pos: Vec2, size: f32, depth: usize) -> Self {
        Self {
            entity: None,
            pos,
            size,
            depth,
        }
    }
}

type MapChunkNode = QuadTree<MapChunkDataNode>;

impl MapChunkNode {
    pub fn new_leaf(pos: Vec2, size: f32, depth: usize) -> Self {
        Self::Data(MapChunkDataNode::new(pos, size, depth))
    }
    pub fn root(size: f32) -> Self {
        Self::Data(MapChunkDataNode::new(Vec2::ZERO, size, 1))
    }
}

#[derive(Debug, Resource)]
struct MapLOD(MapChunkNode);
impl MapLOD {
    pub fn new() -> Self {
        Self(MapChunkNode::root(1.0))
    }
}

struct LODSetter {
    pub view_distance: u16,
    pub ref_distance: u16,
    pub min_precision: u16,
    pub ref_precision: u16,
    pub max_depth: usize,
}

impl LODSetter {
    // Function to convert distance to precision level
    pub fn distance_to_precision(&self, distance: f32) -> u16 {
        let t = (distance / self.ref_distance as f32).clamp(0.0, 1.0);
        let precision_range = (self.ref_precision - self.min_precision) as f32;
        let precision = (1.0 - t) * precision_range + self.min_precision as f32;
        precision.round() as u16
    }
    // Function to determine if a node should be subdivided based on current precision and distance
    pub fn should_subdivide(&self, current_precision: u16, distance: f32) -> bool {
        let target_precision = self.distance_to_precision(distance);
        current_precision < target_precision
    }

    // Function to get size of a node at a given depth
    pub fn get_size(&self, depth: usize) -> f32 {
        let depth = depth as f32;
        self.view_distance as f32 / (2.0_f32.powf(depth) * 2.0)
    }
}

impl TreeSetter for LODSetter {
    type TreeNode = MapChunkNode;

    fn handle_node(&self, root: &mut Self::TreeNode) {
        // type Data = (&mut Self::TreeNode, Vec2, usize);
        let mut stack = Vec::with_capacity(16);
        stack.push((root, Vec2::ZERO, 1));

        while let Some((parent, parent_center, parent_depth)) = stack.pop() {
            if parent_depth > self.max_depth {
                continue;
            }

            // Check if the node should be subdivided
            let parent_distance = parent_center.length();
            if self.should_subdivide(parent_depth as u16, parent_distance) {
                let depth = parent_depth + 1;
                let size = self.get_size(depth);
                let size_half = size / 2.0;
                // Subdivide the node
                let children: [Box<MapChunkNode>; 4] = QUAD_TREE_DIRECTIONS
                    .iter()
                    .map(|direction| {
                        let center = parent_center + *direction * size_half;
                        Box::new(MapChunkNode::new_leaf(center, size, depth))
                    })
                    .collect_vec()
                    .try_into()
                    .unwrap();

                *parent = MapChunkNode::Split(children);

                if let MapChunkNode::Split(children) = parent {
                    for child in children {
                        let (pos, depth) = if let MapChunkNode::Data(ref data) = **child {
                            (data.pos, data.depth)
                        } else {
                            panic!()
                        };

                        stack.push((child, pos, depth));
                    }
                }
            }
        }
    }
}

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

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::X, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z, Color::BLUE);
}

fn render_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_tree: ResMut<MapLOD>,
    max_height: Res<MaxHeight>,
    view_distance: Res<ChunkViewDistance>,
) {
    // let mesh = asset_server
    //     .get_id_handle(MeshType::Plane.into())
    //     .unwrap_or_else(|| asset_server.add(Plane3d::new(*Direction3d::Y).into()));

    let setter = LODSetter {
        view_distance: view_distance.0,
        ref_distance: view_distance.0 / 4,
        min_precision: 2,
        ref_precision: 4,
        max_depth: 4,
    };
    map_tree.0.update_with_setter(&setter);

    info!("chunks: {}", map_tree.0.iter().collect_vec().len());

    let material = asset_server.add(Color::DARK_GREEN.into());
    for chunk in map_tree.0.iter() {
        let transform = Transform::from_xyz(chunk.pos.x, 0.0, chunk.pos.y);

        info!("depth {} -> size {}", chunk.depth, chunk.size);

        let chunk_heights = generate_height_chunk(&max_height, &chunk.pos, chunk.size);

        let render_id = commands
            .spawn((
                Name::new("RenderChunk"),
                PbrBundle {
                    mesh: asset_server.add(create_subdivided_plane(
                        chunk.size,
                        &ChunkHeights::default(),
                        CHUNK_SLICES,
                    )),
                    material: material.clone(),
                    transform,
                    ..default()
                },
            ))
            .id();
    }

    // for x in -5..5 {
    //     for y in -5..5 {
    //         for scale in 1..5 {
    //             let material = asset_server.add(Color::hsl(360.0 / scale as f32, 0.5, 0.5).into());
    //             let transform = Transform::from_xyz(x as f32, scale as f32, y as f32);
    //             let render_id = commands
    //                 .spawn((
    //                     Name::new("RenderChunk"),
    //                     PbrBundle {
    //                         mesh: asset_server.add(create_subdivided_plane(
    //                             1.0 / scale as f32,
    //                             &ChunkHeights::default(),
    //                             CHUNK_SLICES,
    //                         )),
    //                         material: material.clone(),
    //                         transform,
    //                         ..default()
    //                     },
    //                 ))
    //                 .id();
    //         }
    //     }
    // }
}

fn generate_height_chunk(max_height: &Res<MaxHeight>, pos: &Vec2, size: f32) -> ChunkHeights {
    let mut chunk_heights = ChunkHeights::default();
    let max_index = CHUNK_SLICES as f32 - 1.0;
    for z in 0..CHUNK_SLICES {
        for x in 0..CHUNK_SLICES {
            // 0.0 - 1.0
            let sub_chunk_pos = FChunkPos(x as f32 / max_index * size, z as f32 / max_index * size);

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
            let x = (j as f32 / total - 0.5) * size;
            let z = (i as f32 / total - 0.5) * size;

            let mut pos = Vec3::new(x, 0.0, z);
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
    use float_cmp::assert_approx_eq;
    use itertools::Itertools;

    use crate::{FixedTreeNode, LODSetter, MapLOD};

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
    fn map_lod_precision() {
        let tree_creator = LODSetter {
            view_distance: 100,
            ref_distance: 50,
            min_precision: 2,
            ref_precision: 4,
            max_depth: 4,
        };

        let distances = vec![0.0, 1.0, 5.0, 10.0, 20.0, 50.0, 100.0, 500.0, 1000.0];
        let mut last_precision: Option<(f32, u16)> = None;

        for distance in distances {
            let precision = tree_creator.distance_to_precision(distance);
            if let Some(last) = last_precision {
                assert!(
                    precision <= last.1,
                    "(distance: {}, precision: {}) > (distance: {}, precision: {})\nfactor({} / {} =>): {}",
                    distance,
                    precision,
                    last.0,
                    last.1,
										distance, tree_creator.ref_distance,
                    distance / tree_creator.ref_distance as f32,
                );
            }
            last_precision = Some((distance, precision));
        }
    }

    #[test]
    fn map_lod_create() {
        let mut tree = MapLOD::new();
        let setter = LODSetter {
            view_distance: 100,
            ref_distance: 50,
            min_precision: 2,
            ref_precision: 4,
            max_depth: 4,
        };
        tree.0.update_with_setter(&setter);

        let leafs = tree
            .0
            .iter()
            .sorted_unstable_by(|a, b| a.pos.length().total_cmp(&b.pos.length()))
            .collect_vec();

        let min_size = leafs
            .iter()
            .min_by(|a, b| a.size.total_cmp(&b.size))
            .unwrap()
            .size;
        let max_size = leafs
            .iter()
            .max_by(|a, b| a.size.total_cmp(&b.size))
            .unwrap()
            .size;

        let min_distance = leafs
            .iter()
            .min_by(|a, b| a.pos.length().total_cmp(&b.pos.length()))
            .unwrap()
            .pos
            .length();

        let max_distance = leafs
            .iter()
            .max_by(|a, b| a.pos.length().total_cmp(&b.pos.length()))
            .unwrap()
            .pos
            .length();

        println!("Total chunks {}", leafs.len());

        let leaf_data_str = leafs
            .iter()
            .map(|leaf| format!("{} [{}] -> {}", leaf.pos.length(), leaf.depth, leaf.size))
            .join("\n");

        assert_ne!(min_size, max_size);
        assert_ne!(min_distance, max_distance);

        for leaf1 in &leafs {
            for leaf2 in &leafs {
                let pos_ordering = leaf1.pos.length().total_cmp(&leaf2.pos.length());
                let size_ordering = leaf1.size.total_cmp(&leaf2.size);
                if size_ordering.is_ne() {
                    assert!(
                        pos_ordering == size_ordering,
                        "Expected size to be smaller with smaller distance. Received: \n{}",
                        leaf_data_str
                    );
                }
            }
        }
    }
}
