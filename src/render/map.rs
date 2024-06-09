use crate::utils::tree_node;
use bevy::{math::*, prelude::*, render::extract_resource::ExtractResource};
use bevy_easings::Lerp;
use itertools::Itertools;

#[derive(Debug, Clone, Component)]
pub struct MapChunkData {
    pub entity: Option<Entity>,
    pub pos: Vec2,
    pub size: f32,
    pub precision: u8,
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

    pub fn from_setter(setter: &LODSetter) -> Vec<Self> {
        let mut node = MapChunkNode::root(setter.view_distance.into());
        node.update_with_setter(setter);

        node.iter().cloned().collect_vec()
    }
}

pub type MapChunkNode = tree_node::QuadTree<MapChunkData>;

impl MapChunkNode {
    pub fn new_leaf(pos: Vec2, size: f32, precision: u8) -> Self {
        Self::Data(MapChunkData::new(pos, size, precision))
    }
    pub fn root(size: f32) -> Self {
        Self::Data(MapChunkData::new(Vec2::ZERO, size, 1))
    }
}

#[derive(Resource, Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct LODSetter {
    pub view_distance: u16,
    pub factor_distance: u8,
    pub base_precision: u8,
}

impl LODSetter {
    pub fn distance_to_precision(&self, distance: f32) -> u8 {
        let ref_distance = self.view_distance as f32 * self.factor_distance as f32 / 128.0;
        let linear_factor = (ref_distance - distance.min(ref_distance)) / ref_distance;
        let factor = linear_factor * linear_factor;

        1_u8.lerp(&self.base_precision.min(10), &factor)
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

impl tree_node::TreeSetter for LODSetter {
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
                    let center = parent_center + tree_node::QUAD_TREE_DIRECTIONS[i] * size_half;

                    stack.push((child, center, depth));
                }

                continue;
            }

            // If not, then subdivide the node
            let children: [Box<MapChunkNode>; 4] = tree_node::QUAD_TREE_DIRECTIONS
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
                    let center = parent_center + tree_node::QUAD_TREE_DIRECTIONS[i] * size_half;
                    stack.push((child, center, depth));
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_lod_decreases() {
        let tree_creator = LODSetter::new(100, 100, 5);

        let distances = vec![0.0, 1.0, 5.0, 10.0, 20.0, 50.0, 100.0, 500.0, 1000.0];
        let mut last_precision: Option<(f32, u16)> = None;

        for distance in distances {
            let precision = tree_creator.distance_to_precision(distance);
            if let Some(last) = last_precision {
                assert!(
                    u16::from(precision) <= last.1,
                    "precision shouldnt be higher: (distance: {}, precision: {}) > ({}, {})",
                    distance,
                    precision,
                    last.0,
                    last.1,
                );
            }
            last_precision = Some((distance, precision.into()));
        }
    }

    #[test]
    fn map_lod_create() {
        let setter = LODSetter::new(100, 255, 5);
        let leafs = MapChunkData::from_setter(&setter);
        let leafs_sorted = leafs
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
            .map(|leaf| {
                format!(
                    "{} [{}] -> {}",
                    leaf.pos.length(),
                    leaf.precision,
                    leaf.size
                )
            })
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
        let setter = LODSetter::new(100, 255, 4);

        // test on center
        assert_eq!(setter.distance_to_precision(0.0), 4);

        // test on edge
        assert_eq!(setter.distance_to_precision(100.0), 2);
    }

    #[test]
    fn test_get_size() {
        let setter = LODSetter::new(100, 255, 4);

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
