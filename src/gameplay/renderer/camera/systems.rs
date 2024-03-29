use crate::gameplay::data_source_layer::map::components::*;
use crate::gameplay::renderer::camera::components::*;
use crate::gameplay::renderer::renderers::RenderMapApi;
use crate::utils::*;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
pub fn camera_update<R: RenderMapApi + Component>(
    mut camera_query: Query<(
        &Camera,
        &mut Transform,
        Option<Ref<CameraOffset>>,
        Option<Ref<CameraRotation>>,
    )>,
    source_followed_query: Query<Entity, With<SourceCameraFollow>>,
    renderer_query: Query<&R>,
    transform_query: Query<&Transform, Without<Camera>>,
) {
    for renderer in renderer_query.iter() {
        for source_entity in source_followed_query.iter() {
            let target_pos = renderer
                .get_render_item(&source_entity)
                .and_then(|entity| transform_query.get(*entity).ok())
                .map(|transform| Vec3::from_array(transform.translation.to_array()))
                .unwrap_or_default();

            for (camera, mut camera_transform, offset_option, rotation_option) in
                camera_query.iter_mut()
            {
                if !camera.is_active {
                    continue;
                }
                debug!("Handling camera at target {}", target_pos);

                camera_transform.translation.x = target_pos.x;
                camera_transform.translation.y = target_pos.y;
                camera_transform.translation.z = target_pos.z;

                if let Some(offset) = offset_option {
                    camera_transform.translation.x += offset.0;
                    camera_transform.translation.y += offset.1;
                    camera_transform.translation.z += offset.2;
                }

                if let Some(rotation) = rotation_option {
                    let rotation_1 = Quat::from_euler(EULER_ROT, 0.0, 0.0, -rotation.0);
                    let rotation_2 = Quat::from_euler(EULER_ROT, rotation.1, 0.0, 0.0);
                    let combined_rotation = rotation_1 * rotation_2;

                    camera_transform.translate_around(target_pos, combined_rotation);
                    camera_transform.look_at(target_pos, UP);
                }
            }
        }
    }
}

pub fn camera_look_around(
    mut camera_query: Query<(&Camera, &mut CameraRotation)>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for e in motion_evr.read() {
        let time_delta = time.delta_seconds();
        for (camera, mut camera_rotation) in camera_query.iter_mut() {
            if camera.is_active {
                let sensitivity = 20.0;
                camera_rotation.0 += (e.delta.x * time_delta * sensitivity).to_radians();
                camera_rotation.1 += (-e.delta.y * time_delta * sensitivity).to_radians();
            }
        }
    }
}

pub fn camera_follow_rotation(
    mut camera_query: Query<(&Camera, &mut CameraRotation)>,
    target_query: Query<Ref<Rotation>, With<SourceCameraFollow>>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for (camera, mut cam_rotation) in camera_query.iter_mut() {
        if !camera.is_active {
            continue;
        }
        match target_query.get_single() {
            Ok(target_rotation) => {
                if !target_rotation.is_changed() {
                    continue;
                }
                let time_delta = time.delta_seconds();
                cam_rotation.0 = -target_rotation.0.to_euler(EULER_ROT).2;

                for e in motion_evr.read() {
                    cam_rotation.1 += (-e.delta.y * time_delta * 20.0).to_radians()
                }
            }
            Err(err) => error!("{}", err),
        }
    }
}

pub fn camera_zoom(
    mut wheel: EventReader<MouseWheel>,
    mut camera_query: Query<(&Camera, &mut CameraOffset)>,
) {
    for e in wheel.read() {
        for (camera, mut offset) in camera_query.iter_mut() {
            if camera.is_active {
                let mut offset_vec = Vec3::new(offset.0, offset.1, offset.2);
                offset_vec *= (-e.y.clamp(-2.0, 2.0) + 2.0) / 2.0;
                offset.0 = offset_vec.x;
                offset.1 = offset_vec.y;
                offset.2 = offset_vec.z;
            }
        }
    }
}
