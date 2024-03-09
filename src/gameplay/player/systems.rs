use bevy::prelude::*;

use crate::gameplay::map::{
    components::{MapContent, WithPlayerRender},
    renderer::{components::MaterialKey, events::RenderCharacterEvent},
    spawner::MoveSightEvent,
};

use super::{
    components::{MapSpeed, Sight, WSADSteerable},
    events::WSADEvent,
};

pub fn spawn_player(
    mut commands: Commands,
    mut render_character_event: EventWriter<RenderCharacterEvent>,
    mut map_origin_event: EventWriter<MoveSightEvent>,
    map_content_query: Query<Entity, With<MapContent>>,
    with_player_query: Query<Entity, With<WithPlayerRender>>,
) {
    let map_entity = map_content_query.single();
    let sight = 20;
    let player_entity = commands
        .spawn((
            WSADSteerable,
            MapSpeed(15.0),
            Sight(sight),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            },
        ))
        .id();

    commands.entity(map_entity).add_child(player_entity);

    for map_display in with_player_query.iter() {
        map_origin_event.send(MoveSightEvent {
            sight,
            force_render: true,
            map_display,
            ..default()
        });
    }

    render_character_event.send(RenderCharacterEvent {
        parent: player_entity,
        material_key: MaterialKey::Player,
    });
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
    mut items_to_move: Query<(&mut Transform, &MapSpeed, Option<&Sight>), With<WSADSteerable>>,
    mut wsad_event: EventReader<WSADEvent>,
    mut map_origin_event: EventWriter<MoveSightEvent>,
    maps_with_player_query: Query<Entity, With<WithPlayerRender>>,
) {
    for direction in wsad_event.read() {
        for (mut transform, speed, sight) in items_to_move.iter_mut() {
            let delta_pos = direction.0 * speed.0;
            transform.translation += Vec3::new(delta_pos.x, delta_pos.y, 0.0);

            if sight.is_some() {
                let pos = Vec2::new(transform.translation.x, transform.translation.y);
                for map in maps_with_player_query.iter() {
                    map_origin_event.send(MoveSightEvent {
                        pos,
                        delta_pos,
                        sight: sight.unwrap().0,
                        map_display: map,
                        ..default()
                    });
                }
            }
        }
    }
}
