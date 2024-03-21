use std::fmt::Display;

use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::map::utils::lexigraphical_cycle::LexigraphicalCycle;

#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderGroup {
    Gameplay3D,
    PreviewMap2D,
}

#[derive(Component)]
pub struct RenderMap(pub HashMap<u32, Entity>);

#[derive(Component, Debug, Default)]
pub struct CameraOffset(pub f32, pub f32, pub f32);

impl From<CameraOffset> for Vec3 {
    fn from(val: CameraOffset) -> Self {
        Vec3::new(val.0, val.1, val.2)
    }
}

impl From<&CameraOffset> for Vec3 {
    fn from(val: &CameraOffset) -> Self {
        Vec3::new(val.0, val.1, val.2)
    }
}

#[derive(Component, Debug, Default)]
pub struct CameraRotation(pub f32, pub f32, pub f32);

// impl From<CameraRotation> for Quat {
//     fn from(val: CameraRotation) -> Self {
//         // Quat::from_euler(EulerRot::ZXY, -val.0, -val.1, val.2)
// 				Quat::from
//     }
// }

// impl From<&CameraRotation> for Quat {
//     fn from(val: &CameraRotation) -> Self {
//         Quat::from_euler(EulerRot::ZXY, -val.0, -val.1, val.2)
//     }
// }

#[derive(Component)]
pub struct SourceCameraFollow;

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeshType {
    HexMapTile(LexigraphicalCycle<i8, 6>),
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
