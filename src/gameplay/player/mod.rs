use bevy::prelude::*;

use self::events::WSADEvent;

use super::{
    map::{
        events::MoveMapOriginEvent,
        renderer::events::RenderPointEvent,
        utils::{hex_layout::HexLayout, hex_vector::HexVector},
    },
    theme::constants::COLORS,
};

pub mod events;

#[derive(Component)]
pub struct WSADSteerable;

#[derive(Component)]
pub struct MapSpeed(pub f32);

#[derive(Component)]
pub struct Sight(pub u8);

pub fn spawn_player(
    mut commands: Commands,
    mut render_character_event: EventWriter<RenderPointEvent>,
    mut map_origin_event: EventWriter<MoveMapOriginEvent>,
    layout_query: Query<Entity, With<HexLayout>>,
) {
    let layout = layout_query.single();
    let player_entity = commands
        .spawn((
            WSADSteerable,
            MapSpeed(1.0),
            Sight(5),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    commands.entity(layout).add_child(player_entity);

    map_origin_event.send(MoveMapOriginEvent(HexVector::new(0, 0, 0)));

    render_character_event.send(RenderPointEvent {
        parent: player_entity,
        color: COLORS.blue.l300,
        size: 8.0,
    });
}

pub fn move_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wsad_event: EventWriter<WSADEvent>,
) {
    let is_left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let is_right = keyboard.pressed(KeyCode::KeyB) || keyboard.pressed(KeyCode::ArrowRight);
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
    layout_query: Query<&HexLayout>,
    mut items_to_move: Query<(&mut Transform, &MapSpeed, Option<&Sight>), With<WSADSteerable>>,
    mut wsad_event: EventReader<WSADEvent>,
    mut map_origin_event: EventWriter<MoveMapOriginEvent>,
) {
    let layout = layout_query.single();
    for direction in wsad_event.read() {
        for (mut transform, speed, sight) in items_to_move.iter_mut() {
            let vec2 = direction.0 * speed.0;
            let vec3 = Vec3::new(vec2.x, vec2.y, 0.0);
            transform.translation += vec3;

            if sight.is_some() {
                let origin = layout.pixel_to_hex(vec2);
                map_origin_event.send(MoveMapOriginEvent(origin.into()));
            }
        }
    }
}
