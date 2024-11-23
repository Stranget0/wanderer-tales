use super::*;
use crate::{
    game::{map::Terrain, spawn_character, CharacterModel, Jump, LookingAt, Sprinting, Walk},
    prelude::*,
};
use actions::*;
use camera::control::*;
use leafwing_input_manager::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), spawn_player)
        .add_systems(
            Update,
            (handle_look_follow_camera, handle_movement, handle_jump)
                .in_set(GameSet::Update)
                .run_if(in_state(GameState::Playing)),
        );
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>, terrain: Res<Terrain>) {
    let y = terrain.sample(Vec2::ZERO).value;

    spawn_character(
        &mut commands,
        &asset_server,
        &CharacterModel::KnightPlaceholder,
        (
            Name::new("Player"),
            CameraRotationSpeed(45.0_f32.to_radians()),
            CameraRotationController::default(),
            CameraOrbitTarget { zoom: 5.0 },
            crate::game::map::ChunkOrigin,
            PlayerAction::input_bundle(),
            CameraAction::input_bundle(),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, y, 0.0),
                ..default()
            },
        ),
    );
}

fn handle_look_follow_camera(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &CameraRotationController, Option<&mut LookingAt>),
        With<CameraOrbitTarget>,
    >,
) {
    for (entity, camera_controller, looking_at_option) in player_query.iter_mut() {
        let new_looking_at = camera_controller.as_looking_at();
        match looking_at_option {
            Some(mut looking_at) => {
                *looking_at = new_looking_at;
            }
            None => {
                commands.entity(entity).insert(new_looking_at);
            }
        }
    }
}

fn handle_movement(
    mut player_query: Query<(&ActionState<PlayerAction>, &mut Walk, &mut Sprinting)>,
    camera_query: Query<&Transform, With<CameraOrbit>>,
) {
    let camera_transform = camera_query.single();

    for (actions, mut walk, mut sprint) in &mut player_query {
        let axis = actions.axis_pair(&PlayerAction::Move);

        if let Some(movement) = axis.max_normalized() {
            let forward = camera_transform.forward().horizontal().normalize();

            let sideways = forward.cross(Vec3::Y);
            let forward_action = forward * movement.y;
            let sideways_action = sideways * movement.x;

            let modifier = 1.0;
            let direction = forward_action * modifier + sideways_action;

            walk.direction = Dir3::new(direction).ok();
            sprint.requested = actions.pressed(&PlayerAction::Sprint);
        }
    }
}

fn handle_jump(mut player_query: Query<(&ActionState<PlayerAction>, &mut Jump)>) {
    for (actions, mut jump) in &mut player_query {
        jump.requested |= actions.pressed(&PlayerAction::Jump);
    }
}
