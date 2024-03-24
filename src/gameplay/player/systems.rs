use bevy::prelude::*;

use crate::gameplay::{
    map::{
        components::SourceLayout,
        renderer::components::{MaterialType, MeshType, SourceCameraFollow},
        spawner::resources::HexToMapSourceEntity,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::Height,
            hex_vector::{FractionalHexVector, HexVector},
        },
    },
    player::components::HexPositionFractional,
};

use super::{
    components::{
        HexPosition, MapSpeed, PlayerControllable, PlayerRoot, Rotation, Sight, WSADSteerable,
    },
    events::{CharacterMovedEvent, PlayerWithSightSpawnedEvent, WSADEvent},
};

pub fn spawn_player(
    mut commands: Commands,
    mut player_with_sight_event: EventWriter<PlayerWithSightSpawnedEvent>,
    source_layout: Query<Entity, With<SourceLayout>>,
) {
    for layout_entity in source_layout.iter() {
        let pos = HexPositionFractional(FractionalHexVector(0.0, 0.0, 0.0));
        let sight = 2;
        let player_entity = commands
            .spawn((
                WSADSteerable,
                MapSpeed(10.0),
                Sight(sight),
                Height(50),
                PlayerRoot,
                PlayerControllable,
                SourceCameraFollow,
                MeshType::Player,
                MaterialType::Player,
                pos.clone(),
                Rotation(Vec3::ZERO),
            ))
            .id();

        commands.entity(layout_entity).add_child(player_entity);

        player_with_sight_event.send(PlayerWithSightSpawnedEvent {
            sight: Sight(sight),
            pos,
        });
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

type CharacterMoveComponents<'a> = (
    Entity,
    &'a mut HexPositionFractional,
    &'a MapSpeed,
    &'a mut Height,
    Option<&'a Sight>,
);

pub fn move_2d_handle(
    mut items_to_move: Query<CharacterMoveComponents, (With<WSADSteerable>, Without<HexPosition>)>,
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
            for (entity, mut position, speed, mut height, sight_option) in items_to_move.iter_mut()
            {
                let new_position_delta =
                    layout.pixel_to_hex(direction.0) * speed.0 * time.delta_seconds();

                debug!("Move player {:?}", new_position_delta);
                position.0 = &position.0 + &new_position_delta;
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
                    delta_pos: new_position_delta,
                    sight: sight_option.cloned(),
                    is_player_controllable: true,
                });
            }
        }

        character_moved_event.send_batch(events_to_send);
    }
}
