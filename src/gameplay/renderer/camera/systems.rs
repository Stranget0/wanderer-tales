use crate::gameplay::data_source_layer::utils::{Height, HexLayout};
use crate::gameplay::data_source_layer::{map::components::*, player::components::MouseRotatable};
use crate::gameplay::renderer::camera::components::*;
use crate::gameplay::renderer::renderers::RenderMapApi;
use crate::utils::*;
use bevy::render::camera::CameraProjection;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

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

pub fn camera_look_around(
    mut camera_query: Query<(&Camera, &mut Transform)>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for e in motion_evr.read() {
        let time_delta = time.delta_seconds();
        for (camera, mut camera_transform) in camera_query.iter_mut() {
            if camera.is_active {
                camera_transform.translation.x += (e.delta.x * time_delta).to_radians();
                camera_transform.translation.y += (-e.delta.y * time_delta).to_radians();
            }
        }
    }
}

pub fn player_rotation<C: Component, R: RenderMapApi + Component>(
    mut camera_query: Query<(&Camera, &mut Transform), With<C>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut target_query: Query<(
        &mut Rotation,
        &HexPositionFractional,
        &Height,
        &MouseRotatable,
    )>,
    layout: Query<&HexLayout, With<R>>,
    time: Res<Time>,
) {
    for (camera, mut cam_transform) in camera_query.iter_mut() {
        if !camera.is_active {
            continue;
        }

        let time_delta = time.delta_seconds();
        for e in mouse_motion.read() {
            if let Ok((mut target_rotation, position, height, rotatable)) =
                target_query.get_single_mut()
            {
                if let Ok(layout) = layout.get_single() {
                    target_rotation.rotate_right(rotatable.0 * e.delta.x * time_delta);

                    let [x, y] = layout.hex_to_pixel(&position.0).to_array();
                    let z = height.0;
                    let player_pos = Vec3::new(x, y, z.into());
                    let cam_pos = player_pos + Vec3::new(0.0, -3.0, 1.5);

                    cam_transform.translation = cam_pos;
                    cam_transform.rotate_around(
                        player_pos,
                        Quat::from_rotation_z(target_rotation.0.to_euler(EULER_ROT).2),
                    );
                    cam_transform
                        .rotate_around(player_pos, Quat::from_rotation_x(e.delta.y * time_delta));

                    cam_transform.look_at(player_pos, UP);
                }
            }
        }
    }
}

pub fn camera_zoom(
    mut wheel: EventReader<MouseWheel>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    for e in wheel.read() {
        for (camera, mut transform) in camera_query.iter_mut() {
            if !camera.is_active {
                continue;
            }

            transform.translation *= (-e.y.clamp(-2.0, 2.0) + 2.0) / 2.0;
        }
    }
}
