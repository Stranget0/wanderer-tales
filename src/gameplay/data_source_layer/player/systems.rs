use super::{components::*, events::*};
use crate::debug::local_position_gizmo::LocalGizmoSource;
use crate::gameplay::data_source_layer::map::components::*;
use crate::gameplay::data_source_layer::map::resources::HexToMapSourceEntity;
use crate::gameplay::data_source_layer::utils::*;
use crate::gameplay::renderer::camera::components::SourceCameraFollow;
use crate::gameplay::renderer::components::*;
use bevy::{input::mouse::MouseMotion, prelude::*};

pub fn spawn_player(mut commands: Commands, source_layout: Query<Entity, With<SourceLayout>>) {
    for layout_entity in source_layout.iter() {
        let pos = HexPositionFractional(FractionalHexVector(0.0, 0.0, 0.0));
        let sight = 64;
        let player_entity = commands
            .spawn((
                WSADSteerable,
                MapSpeed(50.0),
                Sight(sight),
                Height(50),
                PlayerRoot,
                PlayerControllable,
                SourceCameraFollow,
                MouseRotatable(3.0),
                MeshType::Player,
                MaterialType::Player,
                LocalGizmoSource,
                pos.clone(),
                Rotation::default(),
                Name::new("PlayerSource"),
            ))
            .id();

        commands.entity(layout_entity).add_child(player_entity);
    }
}

pub fn move_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wsad_event: EventWriter<WSADEvent>,
) {
    let is_left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let is_right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let is_up = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
    let is_down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    if is_left || is_right || is_up || is_down {
        let x: f32 = f32::from(is_right) - f32::from(is_left);
        let y = f32::from(is_up) - f32::from(is_down);
        let move_direction = Vec2::new(x, y).normalize();
        wsad_event.send(WSADEvent(move_direction));
    };
}

pub fn move_2d_handle(
    mut items_to_move: Query<
        (
            Entity,
            &mut HexPositionFractional,
            &Rotation,
            &MapSpeed,
            &mut Height,
            Option<&Sight>,
        ),
        (With<WSADSteerable>, Without<HexPosition>),
    >,
    mut wsad_event: EventReader<WSADEvent>,
    mut character_moved_event: EventWriter<CharacterMovedEvent>,
    source_layout: Query<&HexLayout, With<SourceLayout>>,
    hex_to_map_source_entity: Res<HexToMapSourceEntity>,
    height_query: Query<&Height, With<HexPosition>>,
    time: Res<Time>,
) {
    for direction in wsad_event.read() {
        let mut events_to_send: Vec<CharacterMovedEvent> = vec![];
        for layout in source_layout.iter() {
            for (entity, mut position, rotation, speed, mut height, sight_option) in
                items_to_move.iter_mut()
            {
                let rotated_vec = rotation.get_rotated_vec2_x(&direction.0);
                let hex_delta_f = layout.pixel_to_hex(rotated_vec) * speed.0 * time.delta_seconds();

                position.0 = position.0 + hex_delta_f;
                debug!("Move player {:?}", hex_delta_f);

                let k: HexVector = (&position.0).into();
                match hex_to_map_source_entity
                    .0
                    .get(&k)
                    .and_then(|source_entity| height_query.get(*source_entity).ok())
                {
                    Some(hex_height) => {
                        height.0 = hex_height.0;
                    }
                    None => {
                        error!("Could not get hex height at {:?}", k);
                    }
                };

                events_to_send.push(CharacterMovedEvent {
                    source_entity: entity,
                    pos: position.clone(),
                    delta_pos: hex_delta_f,
                    sight: sight_option.cloned(),
                    is_player_controllable: true,
                });
            }
        }

        character_moved_event.send_batch(events_to_send);
    }
}

pub fn rotate_controlled_source(
    mut rotatables_query: Query<(&mut Rotation, &MouseRotatable)>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for motion in motion_evr.read() {
        let delta_seconds = time.delta_seconds();
        for (mut rotation, rotatable) in rotatables_query.iter_mut() {
            rotation.rotate_2d_x(rotatable.0 * -motion.delta.x * delta_seconds);
        }
    }
}
