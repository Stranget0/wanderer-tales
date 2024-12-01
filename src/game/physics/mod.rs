#[cfg(feature = "dev")]
pub mod devtools;

use crate::prelude::*;
use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}

#[derive(PhysicsLayer)]
pub(crate) enum CollisionLayer {
    Player,
    Character,
    Terrain,
    CameraObstacle,
    DetectPlayerSensor,
    DetectCharacterSensor,
}

pub trait CollisionLayersExt {
    fn get_player_colliders() -> CollisionLayers;
    fn get_character_colliders() -> CollisionLayers;
    fn get_terrain_colliders() -> CollisionLayers;
    fn get_camera_obstacle_colliders() -> CollisionLayers;
    fn get_detect_player_sensor_colliders() -> CollisionLayers;
    fn get_detect_character_sensor_colliders() -> CollisionLayers;
}

impl CollisionLayersExt for CollisionLayers {
    fn get_player_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::Player, CollisionLayer::Character],
            [
                CollisionLayer::Terrain,
                CollisionLayer::CameraObstacle,
                CollisionLayer::DetectPlayerSensor,
                CollisionLayer::DetectCharacterSensor,
            ],
        )
    }

    fn get_character_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::Character],
            [
                CollisionLayer::Player,
                CollisionLayer::Terrain,
                CollisionLayer::DetectCharacterSensor,
            ],
        )
    }

    fn get_terrain_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::Terrain],
            [CollisionLayer::Player, CollisionLayer::Character],
        )
    }

    fn get_camera_obstacle_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::CameraObstacle],
            [CollisionLayer::Player, CollisionLayer::Character],
        )
    }
    fn get_detect_player_sensor_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::DetectPlayerSensor],
            [CollisionLayer::Player],
        )
    }
    fn get_detect_character_sensor_colliders() -> CollisionLayers {
        CollisionLayers::new(
            [CollisionLayer::DetectCharacterSensor],
            [CollisionLayer::Character],
        )
    }
}

// impl CollisionLayer {
//     // pub fn get_layers(&self) -> CollisionLayers {
//     //     match self {
//     //         CollisionLayer::Player => CollisionLayers::new(
//     //             [CollisionLayer::Player, CollisionLayer::Character],
//     //             [
//     //                 CollisionLayer::Terrain,
//     //                 CollisionLayer::Character,
//     //                 CollisionLayer::Sensor,
//     //             ],
//     //         ),
//     //         CollisionLayer::Character => CollisionLayers::new(
//     //             [CollisionLayer::Character],
//     //             [
//     //                 CollisionLayer::Terrain,
//     //                 CollisionLayer::Player,
//     //                 CollisionLayer::Character,
//     //             ],
//     //         ),
//     //         CollisionLayer::Terrain => CollisionLayers::new(
//     //             [CollisionLayer::Terrain],
//     //             [CollisionLayer::Player, CollisionLayer::Character],
//     //         ),
//     //         CollisionLayer::CameraObstacle => CollisionLayers::new(
//     //             [CollisionLayer::CameraObstacle],
//     //             [CollisionLayer::Player, CollisionLayer::Character],
//     //         ),
//     //         CollisionLayer::Sensor => CollisionLayers::new(
//     //             [CollisionLayer::Sensor],
//     //             [CollisionLayer::Player, CollisionLayer::Character],
//     //         ),
//     //     }
//     // }
// }

// pub(crate) fn chunk_collision_layers() -> CollisionLayers {
//     CollisionLayers::new(
//         [CollisionLayer::Terrain],
//         [
//             CollisionLayer::Player,
//             CollisionLayer::Character,
//             CollisionLayer::CameraObstacle,
//         ],
//     )
// }
//
// pub(crate) fn character_collision_layers() -> CollisionLayers {
//     CollisionLayers::new(
//         [CollisionLayer::Character],
//         [
//             CollisionLayer::Player,
//             CollisionLayer::Terrain,
//             CollisionLayer::CameraObstacle,
//         ],
//     )
// }
