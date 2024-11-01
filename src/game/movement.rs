//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use crate::prelude::*;
use bevy::input::mouse::MouseMotion;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        (record_movement_controller, record_rotation_controller)
            .in_set(AppSet::RecordInput)
            .run_if(camera_not_locked),
    );

    // Apply movement based on controls.
    app.register_type::<(
        MovementController,
        MovementSpeed,
        RotationController,
        RotationSpeed,
    )>();

    app.add_systems(
        Update,
        (apply_movement, apply_rotation.before(apply_movement)).in_set(AppSet::Update),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController(pub Vec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RotationController(pub Vec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RotationSpeed(pub f32);

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x += 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x -= 1.0;
    }

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 = intent;
    }
}

fn record_rotation_controller(
    mut input: EventReader<MouseMotion>,
    time: Res<Time>,
    mut controller_query: Query<&mut RotationController>,
) {
    let mut intent = Vec2::ZERO;
    for event in input.read() {
        intent.x -= event.delta.x;
        intent.y -= event.delta.y;
    }

    // Apply rotation intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 += intent * time.delta_seconds();
    }
}

fn apply_rotation(
    mut rotation_query: Query<(&RotationController, &RotationSpeed, &mut Transform)>,
) {
    for (rotation, rotation_speed, mut transform) in &mut rotation_query {
        let angle = rotation.0.x.to_radians() * rotation_speed.0;
        let rot = EulerRot::XZY;
        let original_rotation = transform.rotation.to_euler(rot);
        transform.rotation = Quat::from_euler(rot, original_rotation.0, original_rotation.1, angle);
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &MovementSpeed, &mut Transform)>,
) {
    for (movement, movement_speed, mut transform) in &mut movement_query {
        let velocity = transform.rotation
            * movement.0.extend(0.0).xzy()
            * movement_speed.0
            * time.delta_seconds();

        transform.translation += velocity;
    }
}
