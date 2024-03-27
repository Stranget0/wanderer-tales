use std::fmt::Display;

use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderGroup {
    Gameplay3D,
    PreviewMap2D,
}

#[derive(Component)]
pub struct RenderMap(pub HashMap<u32, Entity>);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshType {
    HexMapTile([i8; 6]),
    Player,
    Debug,
}

impl Display for MeshType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MeshType::Debug => "Debug",
            MeshType::HexMapTile(_) => "HexMapTile",
            MeshType::Player => "Player",
        };

        write!(f, "{name}")
    }
}

#[derive(Component, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum MaterialType {
    Beach,
    Mountain,
    Water,
    Grass,
    Forest,
    Player,
    Debug,
}

impl Display for MaterialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MaterialType::Beach => "Beach",
            MaterialType::Mountain => "Mountain",
            MaterialType::Water => "Water",
            MaterialType::Grass => "Grass",
            MaterialType::Forest => "Forest",
            MaterialType::Player => "Player",
            MaterialType::Debug => "Debug",
        };

        write!(f, "{name}")
    }
}
