use crate::game::player_controller::actions::*;
use crate::game::LookingAt;
use crate::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraOrbitTarget {
    pub zoom: f32,
}

#[derive(Component)]
pub struct CameraOrbit;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CameraRotationController(Vec2);

impl CameraRotationController {
    pub fn yaw(&self) -> f32 {
        self.0.x
    }
    pub fn pitch(&self) -> f32 {
        self.0.y
    }
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw()) * Quat::from_rotation_x(self.pitch())
    }
    pub fn offset_direction(&self) -> Dir3 {
        let pitch = self.pitch();
        let yaw = self.yaw();

        Dir3::new(vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            -pitch.cos() * yaw.cos(),
        ))
        .unwrap()
    }

    pub fn forward(&self) -> Dir3 {
        let offset = self.offset_direction();

        Dir3::new(vec3(offset.x, offset.y, -offset.z)).unwrap()
    }

    pub fn as_looking_at(&self) -> LookingAt {
        LookingAt::new(-self.offset_direction())
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CameraRotationSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CameraLerpFactor(pub f32);

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraOrbitTarget>()
        .add_systems(OnEnter(GameState::Playing), spawn_camera_gameplay)
        .add_systems(
            Update,
            (
                record_zoom.in_set(GameSet::RecordInput),
                record_rotation.in_set(GameSet::RecordInput),
                observe_camera_target.in_set(GameSet::Update),
            )
                .run_if(in_state(GameState::Playing)),
        );
}

fn spawn_camera_gameplay(mut commands: Commands) {
    commands.spawn((
        Name::new("Gameplay Camera"),
        CameraOrbit,
        StateScoped(GameState::Playing),
        Camera3dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(1.0, 1.0, 1.0) * 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
}

fn record_zoom(mut observed_target: Query<(&mut CameraOrbitTarget, &ActionState<CameraAction>)>) {
    for (mut target, action) in &mut observed_target {
        let zoom_delta = -action.value(&CameraAction::Zoom);
        target.zoom += target.zoom / 10.0 * zoom_delta;
    }
}

fn record_rotation(
    mut controlled_query: Query<(
        &mut CameraRotationController,
        &CameraRotationSpeed,
        &ActionState<CameraAction>,
    )>,
    time: Res<Time>,
) {
    for (mut controller, rotation_speed, action_state) in controlled_query.iter_mut() {
        let action = action_state.axis_pair(&CameraAction::Orbit);
        controller.0 += action * time.delta_seconds() * rotation_speed.0;
        controller.0 %= std::f32::consts::PI * 2.0;
    }
}

fn observe_camera_target(
    observed_target: Query<
        (&CameraOrbitTarget, &CameraRotationController, &Transform),
        Without<CameraOrbit>,
    >,
    mut camera_query: Query<&mut Transform, With<CameraOrbit>>,
) {
    for mut camera in &mut camera_query {
        match observed_target.iter().next() {
            Some((orbit_target, rotation_controller, observed_transform)) => {
                let zoom = orbit_target.zoom;
                let direction = rotation_controller.offset_direction();

                camera.translation = direction * zoom + observed_transform.translation;
                camera.look_at(observed_transform.translation, Vec3::Y);
            }
            None => {
                warn!("No matching target to orbit around");
                camera.translation.z = 10.0
            }
        }
    }
}
