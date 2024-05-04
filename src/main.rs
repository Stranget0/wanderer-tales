use std::fmt::Debug;

use bevy::{
    math::*,
    pbr::{wireframe::WireframeConfig, ExtendedMaterial},
    prelude::*,
    render::{
        extract_resource::ExtractResource,
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
use wanderer_tales::{debug::fps_counter::FPSPlugin, utils::WorldAlignedExtension};

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
        .insert_resource(MaxHeight(100))
        .insert_resource(LODSetter::new(1500, 100, 10))
        .insert_resource(MapLOD::new())
        .register_type::<LODSetter>()
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, WorldAlignedExtension>,
        >::default())
        .add_systems(Startup, (render_chunks))
        .add_systems(Update, (render_chunks, draw_gizmos))
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
    pub fn is_leaf(&self) -> bool {
        match self {
            FixedTreeNode::Data(_) => true,
            FixedTreeNode::Split(_) => false,
        }
    }

    pub fn iter(&self) -> FixedTreeNodeIterator<'_, SIZE, T> {
        FixedTreeNodeIterator { stack: vec![self] }
    }

    pub fn iter_mut(&mut self) -> FixedTreeNodeIteratorMut<'_, SIZE, T> {
        FixedTreeNodeIteratorMut { stack: vec![self] }
    }

    pub fn update_with_setter<S: TreeSetter<TreeNode = Self>>(&mut self, setter: &S) {
        setter.handle_node(self);
    }
}

struct FixedTreeNodeIterator<'a, const SIZE: usize, T> {
    stack: Vec<&'a FixedTreeNode<SIZE, T>>,
}
struct FixedTreeNodeIteratorMut<'a, const SIZE: usize, T> {
    stack: Vec<&'a mut FixedTreeNode<SIZE, T>>,
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

impl<'a, const SIZE: usize, T> Iterator for FixedTreeNodeIteratorMut<'a, SIZE, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = match self.stack.pop() {
                Some(n) => n,
                None => return None,
            };

            match node {
                FixedTreeNode::Data(ref mut data) => return Some(data),
                FixedTreeNode::Split(ref mut children) => {
                    for child in children.iter_mut() {
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
struct MapChunkData {
    entity: Option<Entity>,
    pos: Vec2,
    size: f32,
    precision: u8,
}

impl MapChunkData {
    pub fn new(pos: Vec2, size: f32, precision: u8) -> Self {
        Self {
            entity: None,
            pos,
            size,
            precision,
        }
    }
}

type MapChunkNode = QuadTree<MapChunkData>;

impl MapChunkNode {
    pub fn new_leaf(pos: Vec2, size: f32, precision: u8) -> Self {
        Self::Data(MapChunkData::new(pos, size, precision))
    }
    pub fn root(size: f32) -> Self {
        Self::Data(MapChunkData::new(Vec2::ZERO, size, 1))
    }
}

#[derive(Debug, Resource)]
struct MapLOD(MapChunkNode);
impl MapLOD {
    pub fn new() -> Self {
        Self(MapChunkNode::root(1.0))
    }
}

#[derive(Resource, Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
struct LODSetter {
    pub view_distance: u16,
    pub factor_distance: u8,
    pub base_precision: u8,
}

impl LODSetter {
    pub fn distance_to_precision(&self, distance: f32) -> u8 {
        let ref_distance = self.view_distance as f32 * self.factor_distance as f32 / 128.0;
        let linear_factor = (ref_distance - distance.min(ref_distance)) / ref_distance;
        let factor = linear_factor * linear_factor;

        1.lerp(&self.base_precision.min(10), &factor)
    }

    pub fn get_size(&self, precision: u8) -> f32 {
        let depth = (precision - 1) as f32;

        self.view_distance as f32 / (2.0_f32.powf(depth))
    }

    pub fn new(view_distance: u16, factor_distance: u8, base_precision: u8) -> Self {
        Self {
            view_distance,
            factor_distance,
            base_precision,
        }
    }
}

impl TreeSetter for LODSetter {
    type TreeNode = MapChunkNode;

    fn handle_node(&self, root: &mut Self::TreeNode) {
        let mut stack = Vec::with_capacity(16);
        stack.push((root, Vec2::ZERO, 1));

        while let Some((parent, parent_center, parent_depth)) = stack.pop() {
            let parent_size = self.get_size(parent_depth);
            let parent_distance = parent_center.length();

            // Check if the node should not be split
            if parent_depth >= self.distance_to_precision(parent_distance) {
                // If so, then update the leaf
                *parent = MapChunkNode::new_leaf(parent_center, parent_size, parent_depth);

                continue;
            }

            let depth = parent_depth + 1;
            let size = self.get_size(depth);
            let size_half = size / 2.0;

            // Check if it is already split
            if let MapChunkNode::Split(children) = parent {
                // Traverse
                for (i, child) in children.iter_mut().enumerate() {
                    let center = parent_center + QUAD_TREE_DIRECTIONS[i] * size_half;

                    stack.push((child, center, depth));
                }

                continue;
            }

            // If not, then subdivide the node
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

            // Traverse
            if let MapChunkNode::Split(children) = parent {
                for (i, child) in children.iter_mut().enumerate() {
                    let center = parent_center + QUAD_TREE_DIRECTIONS[i] * size_half;
                    stack.push((child, center, depth));
                }
            };
        }
    }
}

#[derive(Debug, Component)]
struct MapChunk;

#[derive(Debug, Component)]
struct ChunkHeights([u8; CHUNK_ITEM_COUNT]);

#[derive(Debug, Component)]
struct RenderChunkLink(pub Vec<Entity>);

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

fn draw_gizmos(mut gizmos: Gizmos, lod: Res<LODSetter>) {
    let factor = (lod.view_distance / 2) as f32;
    gizmos.arrow(Vec3::ZERO, Vec3::X * factor, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * factor, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * factor, Color::BLUE);
}

fn render_chunks(
    mut commands: Commands,
    mut map_tree: ResMut<MapLOD>,
    asset_server: Res<AssetServer>,
    max_height: Res<MaxHeight>,
    chunk_setter: Res<LODSetter>,
    chunks: Query<Entity, With<MapChunk>>,
) {
    if !chunk_setter.is_added() && !chunk_setter.is_changed() {
        return;
    }

    for entity in chunks.iter() {
        commands.entity(entity).despawn();
    }

    map_tree.0.update_with_setter(chunk_setter.as_ref());
    let material = asset_server.add(Color::DARK_GREEN.into());
    for chunk in map_tree.0.iter_mut() {
        let transform = Transform::from_xyz(chunk.pos.x, 0.0, chunk.pos.y);
        let chunk_heights = generate_height_chunk(&max_height, &chunk.pos, chunk.size);

        spawn_single_chunk(
            &mut commands,
            &asset_server,
            chunk,
            chunk_heights,
            &material,
            transform,
        );
    }
}

fn spawn_single_chunk(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    chunk: &mut MapChunkData,
    chunk_heights: ChunkHeights,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) {
    let render_id = commands
        .spawn((
            Name::new(format!("Chunk-{}", chunk.precision)),
            MapChunk,
            PbrBundle {
                mesh: asset_server.add(create_subdivided_plane(
                    chunk.size,
                    &chunk_heights,
                    CHUNK_SLICES,
                )),
                material: material.clone(),
                transform,
                ..default()
            },
        ))
        .id();

    chunk.entity = Some(render_id);
}

fn generate_height_chunk(max_height: &Res<MaxHeight>, offset: &Vec2, size: f32) -> ChunkHeights {
    let mut chunk_heights = ChunkHeights::default();
    let max_index = CHUNK_SLICES as f32 - 1.0;
    for z in 0..CHUNK_SLICES {
        for x in 0..CHUNK_SLICES {
            let pos = Vec2::new(
                x as f32 / max_index * size + offset.x,
                z as f32 / max_index * size + offset.y,
            );
            let height_f = simplex_noise_2d_seeded(pos, 0.0) / 2.0 + 0.5;

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
        for (i, node) in tree.iter().collect_vec().iter().rev().enumerate() {
            assert_eq!(i as i32, **node);
        }
    }

    #[test]
    fn map_lod_decreases() {
        let tree_creator = LODSetter::new(100, 100, 5);

        let distances = vec![0.0, 1.0, 5.0, 10.0, 20.0, 50.0, 100.0, 500.0, 1000.0];
        let mut last_precision: Option<(f32, u16)> = None;

        for distance in distances {
            let precision = tree_creator.distance_to_precision(distance);
            if let Some(last) = last_precision {
                assert!(
                    precision <= last.1,
                    "precision shouldnt be higher: (distance: {}, precision: {}) > (distance: {}, precision: {})\nfactor({} / {} =>): {}",
                    distance,
                    precision,
                    last.0,
                    last.1,
                );
            }
            last_precision = Some((distance, precision));
        }
    }

    #[test]
    fn map_lod_create() {
        let mut tree = MapLOD::new();
        let mut setter = LODSetter::new(100, 100, 5);
        tree.0.update_with_setter(&mut setter);

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

    #[test]
    fn test_distance_to_precision() {
        let setter = LODSetter::new(100, 50, 4);

        // Test when distance is equal to reference distance
        assert_eq!(setter.distance_to_precision(50.0), 4);

        // Test when distance is less than reference distance
        assert_eq!(setter.distance_to_precision(25.0), 16);

        // Test when distance is greater than reference distance
        assert_eq!(setter.distance_to_precision(75.0), 4);

        // Test when distance is greater than view distance
        assert_eq!(setter.distance_to_precision(150.0), 4);
    }

    #[test]
    fn test_get_size() {
        let setter = LODSetter::new(100, 50, 4);

        // Test precision 1
        assert_eq!(setter.get_size(1), 100.0);

        // Test precision 2
        assert_eq!(setter.get_size(2), 50.0);

        // Test precision 3
        assert_eq!(setter.get_size(3), 25.0);

        // Test precision 4
        assert_eq!(setter.get_size(4), 12.5);
    }
}
