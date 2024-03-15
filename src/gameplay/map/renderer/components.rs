use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderGroup {
    Gameplay3D,
    PreviewMap2D,
}

#[derive(Component)]
pub struct PlayerRender;

#[derive(Component)]
pub struct CameraFollowTarget;

#[derive(Component)]
pub struct RenderMap(pub HashMap<u32, Entity>);

#[derive(Component)]
pub enum CameraSetting {
    Follow2D(Transform),
    Follow3D(Transform),
}

impl CameraSetting {
    fn attach_to_followed(mut commands: Commands, followed: Entity, camera_entity: Entity) {
        commands.entity(followed).add_child(camera_entity);
    }
    pub fn handle_camera(&self, mut commands: Commands, followed: Entity) -> Entity {
        let camera_entity = match self {
            CameraSetting::Follow2D(transform) => commands
                .spawn(Camera2dBundle {
                    transform: *transform,
                    ..default()
                })
                .id(),
            CameraSetting::Follow3D(transform) => commands
                .spawn(Camera3dBundle {
                    transform: *transform,
                    ..default()
                })
                .id(),
        };

        Self::attach_to_followed(commands, followed, camera_entity);

        camera_entity
    }
}
