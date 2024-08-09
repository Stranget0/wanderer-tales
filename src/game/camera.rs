use crate::{game::movement::*, screen::Screen};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::AppSet;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraOrbitTarget {
    pub zoom: f32,
}

#[derive(Component)]
pub struct CameraOrbit;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraOrbitTarget>()
        .add_systems(OnEnter(Screen::Playing), spawn_camera_gameplay)
        .add_systems(
            Update,
            (
                handle_zoom.in_set(AppSet::RecordInput),
                observe_camera_target.in_set(AppSet::Update),
            )
                .run_if(in_state(Screen::Playing)),
        );
}

fn spawn_camera_gameplay(mut commands: Commands) {
    commands.spawn((
        Name::new("Gameplay Camera"),
        CameraOrbit,
        StateScoped(Screen::Playing),
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

fn handle_zoom(
    mut wheel: EventReader<MouseWheel>,
    mut observed_target: Query<&mut CameraOrbitTarget>,
) {
    for event in wheel.read() {
        for mut target in &mut observed_target {
            match event.unit {
                MouseScrollUnit::Line => target.zoom += event.y * 0.1,
                MouseScrollUnit::Pixel => target.zoom += event.y,
            }
        }
    }
}

fn observe_camera_target(
    observed_target: Query<
        (
            &CameraOrbitTarget,
            &RotationController,
            &RotationSpeed,
            &Transform,
        ),
        Without<CameraOrbit>,
    >,
    mut camera_query: Query<&mut Transform, With<CameraOrbit>>,
) {
    for mut camera in &mut camera_query {
        match observed_target.iter().next() {
            Some((options, rotation, rotation_speed, transform)) => {
                let rotation_x = rotation.0.y.to_radians() * rotation_speed.0;

                let quat_x = Quat::from_rotation_x(rotation_x);
                let offset = -Vec3::Z * options.zoom;

                camera.translation = transform.translation
                    + transform.rotation * Quat::from_rotation_x(-rotation_x) * offset;

                camera.rotation =
                    transform.rotation * Quat::from_rotation_y(180.0_f32.to_radians()) * quat_x;

                info!("Camera: {:?}", camera.translation);
            }
            None => {
                warn!("No matching target to orbit around");
                camera.translation.z = 10.0
            }
        }
    }
}
