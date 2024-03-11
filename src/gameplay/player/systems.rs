use bevy::prelude::*;

use crate::gameplay::{
    map::{
        components::SourceLayout,
        renderer::{components::RenderGroup, events::RenderCharacterEvent, utils::MaterialKey},
        utils::{hex_layout::HexLayout, hex_vector::FractionalHexVector},
    },
    player::components::{HexPositionFractional, HexPositionFractionalDelta},
};

use super::{
    components::{MapSpeed, PlayerControllable, PlayerRoot, Sight, WSADSteerable},
    events::{CharacterMovedEvent, PlayerWithSightSpawnedEvent, WSADEvent},
};

const PLAYER_RENDER_GROUPS: [RenderGroup; 2] = [RenderGroup::Gameplay3D, RenderGroup::PreviewMap2D];

pub fn spawn_player(
    mut commands: Commands,
    mut render_character_event: EventWriter<RenderCharacterEvent>,
    mut player_with_sight_event: EventWriter<PlayerWithSightSpawnedEvent>,
    source_layout: Query<Entity, With<SourceLayout>>,
) {
    for layout_entity in source_layout.iter() {
        let pos = HexPositionFractional(FractionalHexVector(0.0, 0.0, 0.0));
        let sight = 3;
        let player_entity = commands
            .spawn((
                WSADSteerable,
                MapSpeed(2.0),
                Sight(sight),
                PlayerRoot,
                PlayerControllable,
                pos.clone(),
                HexPositionFractionalDelta::default(),
            ))
            .id();

        commands.entity(layout_entity).add_child(player_entity);

        render_character_event.send(RenderCharacterEvent {
            character_entity: player_entity,
            material_key: MaterialKey::Player,
            position: pos.clone(),
            render_groups: PLAYER_RENDER_GROUPS.to_vec(),
        });

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
    &'a mut HexPositionFractionalDelta,
    &'a MapSpeed,
    Option<&'a Sight>,
);

pub fn move_2d_handle(
    mut items_to_move: Query<
        CharacterMoveComponents,
        (With<WSADSteerable>, With<PlayerControllable>),
    >,
    mut wsad_event: EventReader<WSADEvent>,
    mut character_moved_event: EventWriter<CharacterMovedEvent>,
    source_layout: Query<&HexLayout, With<SourceLayout>>,
) {
    for direction in wsad_event.read() {
        let mut events_to_send: Vec<CharacterMovedEvent> = vec![];
        for layout in source_layout.iter() {
            for (entity, mut position, mut position_delta, speed, sight_option) in
                items_to_move.iter_mut()
            {
                let new_position_delta = layout.pixel_to_hex(direction.0) * speed.0;
                position_delta.0 = new_position_delta.clone();
                position.0 = &position.0 + &new_position_delta;

                events_to_send.push(CharacterMovedEvent {
                    character_source: entity,
                    pos: position.clone(),
                    delta_pos: position_delta.clone(),
                    sight: sight_option.cloned(),
                    is_player_controllable: true,
                });
            }
        }

        character_moved_event.send_batch(events_to_send);
    }
}
