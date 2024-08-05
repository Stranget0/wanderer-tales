use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::AppSet;

use super::prelude::MovementController;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraTarget {
    pub zoom: f32,
}

#[derive(Component)]
pub struct CameraObserver;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraTarget>();
    app.add_systems(Update, observe_camera_target.in_set(AppSet::Update));
    app.add_systems(Update, handle_zoom.in_set(AppSet::RecordInput));
}

fn observe_camera_target(
    time: Res<Time>,
    observed_target: Query<
        (&Transform, &CameraTarget, &MovementController),
        Without<CameraObserver>,
    >,
    mut camera_query: Query<&mut Transform, With<CameraObserver>>,
) {
    for mut camera in &mut camera_query {
        match observed_target.iter().next() {
            Some((transform, options, controller)) => {
                let rotation_x = controller.rotation.y.to_radians();
                info!("rotation_x: {:?}", rotation_x);

                let quat_x = Quat::from_rotation_x(rotation_x);
                let offset = -Vec3::Z * options.zoom;

                camera.translation = transform.translation
                    + transform.rotation * Quat::from_rotation_x(-rotation_x) * offset;
                camera.rotation =
                    transform.rotation * Quat::from_rotation_y(180.0_f32.to_radians()) * quat_x;
            }
            None => camera.translation.z = 10.0,
        }
    }
}

fn handle_zoom(mut wheel: EventReader<MouseWheel>, mut observed_target: Query<&mut CameraTarget>) {
    for event in wheel.read() {
        for mut target in &mut observed_target {
            match event.unit {
                MouseScrollUnit::Line => target.zoom += event.y * 0.1,
                MouseScrollUnit::Pixel => target.zoom += event.y,
            }
        }
    }
}
