use bevy::prelude::*;

pub const CHUNK_SLICES: usize = 2;
pub const CHUNK_ITEM_COUNT: usize = CHUNK_SLICES * CHUNK_SLICES;

pub const QUAD_TREE_DIRECTIONS: [Vec2; 4] = [
    Vec2::new(-1.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
];

#[derive(Debug, Clone, PartialEq)]
pub enum FixedTreeNode<const SIZE: usize, T> {
    Data(T),
    Split([Box<FixedTreeNode<SIZE, T>>; SIZE]),
}

pub trait TreeSetter {
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

pub struct FixedTreeNodeIterator<'a, const SIZE: usize, T> {
    pub stack: Vec<&'a FixedTreeNode<SIZE, T>>,
}

pub struct FixedTreeNodeIteratorMut<'a, const SIZE: usize, T> {
    pub stack: Vec<&'a mut FixedTreeNode<SIZE, T>>,
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

pub type OcTree<T> = FixedTreeNode<8, T>;

pub type QuadTree<T> = FixedTreeNode<4, T>;

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

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
}
