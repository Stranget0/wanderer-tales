//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        (record_movement_controller, record_rotation_controller).in_set(AppSet::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<Movement>();
    app.add_systems(Update, (apply_movement).in_set(AppSet::Update));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController {
    pub movement: Vec2,
    pub rotation: Vec2,
}

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
        controller.movement = intent;
    }
}

fn record_rotation_controller(
    mut input: EventReader<MouseMotion>,
    mut controller_query: Query<&mut MovementController>,
) {
    let mut intent = Vec2::ZERO;
    for event in input.read() {
        intent.x -= event.delta.x;
        intent.y -= event.delta.y;
    }

    // Apply rotation intent to controllers.
    for mut controller in &mut controller_query {
        controller.rotation += intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?u
    /// Note that physics engines may use different unit/pixel ratios.
    pub speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &Movement, &mut Transform)>,
) {
    for (controller, movement, mut transform) in &mut movement_query {
        let angle = controller.rotation.x.to_radians();
        let rot = EulerRot::XZY;
        let original_rotation = transform.rotation.to_euler(rot);
        transform.rotation = Quat::from_euler(rot, original_rotation.0, original_rotation.1, angle);

        let velocity = transform.rotation
            * controller.movement.extend(0.0).xzy()
            * movement.speed
            * time.delta_seconds();

        transform.translation += velocity;
    }
}
