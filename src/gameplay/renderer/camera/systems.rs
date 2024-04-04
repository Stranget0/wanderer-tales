use crate::gameplay::data_source_layer::map::components::*;
use crate::gameplay::data_source_layer::utils::{Height, HexLayout};
use crate::gameplay::renderer::camera::components::*;
use crate::gameplay::renderer::renderers::RenderMapApi;
use crate::utils::*;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use super::states::CameraMode;

const ROTATION_SENSITIVITY: f32 = 1.0;
const ZOOM_SENSITIVITY: f32 = 2.0;

// pub fn camera_update<R: RenderMapApi + Component>(
//     mut camera_query: Query<(&Camera, &mut Transform)>,
//     source_followed_query: Query<Entity, With<SourceCameraFollow>>,
//     renderer_query: Query<&R>,
//     transform_query: Query<&Transform, Without<Camera>>,
// ) {
//     for renderer in renderer_query.iter() {
//         for source_entity in source_followed_query.iter() {
//             let target_pos = renderer
//                 .get_render_item(&source_entity)
//                 .and_then(|entity| transform_query.get(*entity).ok())
//                 .map(|transform| Vec3::from_array(transform.translation.to_array()))
//                 .unwrap_or_default();

//             for (camera, mut camera_transform, offset_option, rotation_option) in
//                 camera_query.iter_mut()
//             {
//                 if !camera.is_active {
//                     continue;
//                 }
//                 debug!("Handling camera at target {}", target_pos);

//                 camera_transform.translation.x = target_pos.x;
//                 camera_transform.translation.y = target_pos.y;
//                 camera_transform.translation.z = target_pos.z;

//                 if let Some(offset) = offset_option {
//                     camera_transform.translation.x += offset.0;
//                     camera_transform.translation.y += offset.1;
//                     camera_transform.translation.z += offset.2;
//                 }

//                 if let Some(rotation) = rotation_option {
//                     let rotation_1 = Quat::from_euler(EULER_ROT, 0.0, 0.0, -rotation.0);
//                     let rotation_2 = Quat::from_euler(EULER_ROT, rotation.1, 0.0, 0.0);
//                     let combined_rotation = rotation_1 * rotation_2;

//                     camera_transform.translate_around(target_pos, combined_rotation);
//                     camera_transform.look_at(target_pos, UP);
//                 }
//             }
//         }
//     }
// }

// pub fn camera_look_around(
//     mut camera_query: Query<(&Camera, &mut Transform)>,
//     mut motion_evr: EventReader<MouseMotion>,
//     time: Res<Time>,
// ) {
//     for e in motion_evr.read() {
//         let time_delta = time.delta_seconds();
//         for (camera, mut camera_transform) in camera_query.iter_mut() {
//             if camera.is_active {
//                 camera_transform.translation.x += (e.delta.x * time_delta).to_radians();
//                 camera_transform.translation.y += (-e.delta.y * time_delta).to_radians();
//             }
//         }
//     }
// }

// pub fn player_rotation<C: Component, R: RenderMapApi + Component>(
//     mut camera_query: Query<(&Camera, &mut Transform, &CameraSlide), With<C>>,
//     mut target_query: Query<(
//         &mut Rotation,
//         &HexPositionFractional,
//         &Height,
//         &MouseRotatable,
//     )>,
//     layout: Query<&HexLayout, With<R>>,
// ) {
//     for (camera, mut cam_transform, cam_slide) in camera_query.iter_mut() {
//         if !camera.is_active {
//             continue;
//         }

//         let time_delta = time.delta_seconds();

//         if let Ok((mut target_rotation, position, height, rotatable)) =
//             target_query.get_single_mut()
//         {
//             if let Ok(layout) = layout.get_single() {
//                 let delta = mouse_motion
//                     .read()
//                     .last()
//                     .map(|e| e.delta)
//                     .unwrap_or_default();

//                 // Rotate source item
//                 target_rotation.rotate_right(rotatable.0 * delta.x * time_delta);
//                 let player_pos = get_3d_position(layout, position, height);

//                 // Move camera
//                 let cam_pos = cam_slide.0.get_value().0 + player_pos;
//                 cam_transform.translation = cam_pos;

//                 // Rotate camera
//                 let new_x_rotation = *last_x_rotation + rotatable.0 * delta.y * time_delta;
//                 cam_transform.rotate_around(
//                     player_pos,
//                     Quat::from_rotation_z(target_rotation.0.to_euler(EULER_ROT).2)
//                         * Quat::from_rotation_x(new_x_rotation),
//                 );
//                 *last_x_rotation = new_x_rotation;

//                 cam_transform.look_at(player_pos, UP);
//             }
//         }
//     }
// }

pub fn camera_zoom(
    mut wheel: EventReader<MouseWheel>,
    mut camera_query: Query<(&Camera, &mut CameraSlide)>,
    time: Res<Time>,
) {
    for e in wheel.read() {
        for (camera, mut camera_slide) in camera_query.iter_mut() {
            if !camera.is_active {
                continue;
            }
            let time_delta = time.delta_seconds();
            let delta = -e.y.clamp(-2.0, 2.0) / 2.0 * time_delta * ZOOM_SENSITIVITY;
            camera_slide.0.step_factor(delta);
        }
    }
}

pub fn camera_rotation(
    mut motion: EventReader<MouseMotion>,
    mut camera_query: Query<(&Camera, &mut CameraRotationRange)>,
    camera_mode: Res<State<CameraMode>>,
    time: Res<Time>,
) {
    for e in motion.read() {
        for (camera, mut camera_rotation) in camera_query.iter_mut() {
            if !camera.is_active {
                continue;
            }

            let delta_seconds = time.delta_seconds();
            if camera_mode.get() == &CameraMode::LookAround {
                let delta_x = e.delta.x * delta_seconds * ROTATION_SENSITIVITY;
                camera_rotation.0.step_factor(delta_x);
            }

            let delta_y = -e.delta.y * delta_seconds * ROTATION_SENSITIVITY;
            camera_rotation.1.step_factor(delta_y);
        }
    }
}
pub fn camera_rotation_reset(mut camera_query: Query<&mut CameraRotationRange>) {
    for mut camera_rotation in camera_query.iter_mut() {
        camera_rotation.0.set_factor(0.5);
        camera_rotation.1.set_factor(0.5);
    }
}

pub fn camera_follow<C: Component, R: RenderMapApi + Component>(
    mut commands: Commands,
    camera_query: Query<Entity, With<C>>,
    source_followed_query: Query<Entity, With<SourceCameraFollow>>,
    transform_query: Query<&Transform, (Without<Camera>, Without<SourceCameraFollow>)>,
    renderer_query: Query<&R>,
) {
    for camera_entity in camera_query.iter() {
        match source_followed_query
            .get_single()
            .ok()
            .and_then(|source_entity| {
                renderer_query
                    .get_single()
                    .ok()
                    .and_then(|r| r.get_render_item(&source_entity))
            })
            .and_then(|render_entity| transform_query.get(*render_entity).ok())
        {
            Some(followed_transform) => {
                commands
                    .entity(camera_entity)
                    .insert(CameraOffset(*followed_transform));
                debug!("Camera follow")
            }
            None => debug!("Could not follow entity"),
        };
    }
}

pub fn camera_transform(
    mut camera_query: Query<
        (
            &CameraRotationRange,
            &CameraSlide,
            Option<&CameraOffset>,
            &mut Transform,
        ),
        With<Camera>,
    >,
    mut gizmos: Gizmos,
    mut last_rotation: Local<Quat>,
) {
    for (rotation_range, camera_slide, offset_option, mut transform) in camera_query.iter_mut() {
        let followed_transform = offset_option.unwrap_or(&CameraOffset::default()).0;
        let camera_slide_offset = camera_slide.0.get_value().0;
        let rotation_point = followed_transform.translation;
        gizmos.arrow(rotation_point, rotation_point + UP, Color::MIDNIGHT_BLUE);
        let rotation = Quat::from_euler(
            EULER_ROT,
            0.0,
            0.0,
            rotation_range.0.get_value().to_radians()
                + followed_transform.rotation.to_euler(EULER_ROT).2,
        ) * Quat::from_euler(
            EULER_ROT,
            rotation_range.1.get_value().to_radians(),
            0.0,
            0.0,
        );
        if *last_rotation != rotation {
            // info!(
            //     "{}",
            //     rotation.to_euler(EULER_ROT).2 - followed_transform.rotation.to_euler(EULER_ROT).2
            // );
            info!("{}", rotation_range.1.get_value());
            *last_rotation = rotation;
        }
        transform.translation = followed_transform.translation + camera_slide_offset;
        transform.rotate_around(rotation_point, rotation);
        transform.look_at(rotation_point, UP);
    }
}

pub fn followed_rotation(
    mut followed_query: Query<&mut Rotation, With<SourceCameraFollow>>,
    mut motion: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for e in motion.read() {
        if let Ok(mut rotation) = followed_query.get_single_mut() {
            let delta_seconds = time.delta_seconds();
            let delta_z = -e.delta.x * delta_seconds * ROTATION_SENSITIVITY;
            rotation.0 = Quat::from_euler(EULER_ROT, 0.0, 0.0, delta_z) * rotation.0;
        }
    }
}

fn get_3d_position(layout: &HexLayout, position: &HexPositionFractional, height: &Height) -> Vec3 {
    let [x, y] = layout.hex_to_pixel(&position.0).to_array();
    let z = height.0;
    let player_pos = Vec3::new(x, y, z.into());
    player_pos
}
