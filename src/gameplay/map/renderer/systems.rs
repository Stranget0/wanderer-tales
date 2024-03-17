use crate::gameplay::{
    map::utils::{hex_layout::HexLayout, hex_map_item::Height, hex_vector::FractionalHexVector},
    player::components::{HexPosition, HexPositionFractional, HexPositionFractionalDelta},
};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use super::{
    components::{CameraOffset, CameraRotation, MaterialType, MeshType, SourceCameraFollow},
    renderers::traits::{CreateRenderBundle, RenderMapApi},
};

pub(crate) fn render_static_map_items<
    T: Bundle,
    R: CreateRenderBundle<T> + RenderMapApi + Component,
>(
    mut commands: Commands,
    render_type_query: Query<(Entity, &HexPosition, &Height, &MeshType, &MaterialType)>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
) {
    for (source_entity, position, height, mesh_type, material_type) in render_type_query.iter() {
        for (layout_entity, layout, renderer) in layout_query.iter_mut() {
            if renderer.get_render_item(&source_entity).is_some() {
                continue;
            }
            let pos_2d = layout.hex_to_pixel(&FractionalHexVector::from(&position.0));
            let pos = Vec3::new(pos_2d.x, pos_2d.y, height.0.into());

            spawn_render_item(
                &mut commands,
                renderer,
                pos,
                material_type,
                mesh_type,
                source_entity,
                layout_entity,
            );
        }
    }
}

pub(crate) fn render_map_items<T: Bundle, R: CreateRenderBundle<T> + RenderMapApi + Component>(
    mut commands: Commands,
    render_type_query: Query<(
        Entity,
        &HexPositionFractional,
        &Height,
        &MeshType,
        &MaterialType,
    )>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
) {
    for (source_entity, position, height, mesh_type, material_type) in render_type_query.iter() {
        for (layout_entity, layout, renderer) in layout_query.iter_mut() {
            if renderer.get_render_item(&source_entity).is_some() {
                continue;
            }
            let pos_2d = layout.hex_to_pixel(&position.0);
            let pos = Vec3::new(pos_2d.x, pos_2d.y, height.0.into());

            spawn_render_item(
                &mut commands,
                renderer,
                pos,
                material_type,
                mesh_type,
                source_entity,
                layout_entity,
            );
        }
    }
}

pub(crate) fn clean_render_items<R: RenderMapApi + Component>(
    mut commands: Commands,
    source_item_query: Query<(Entity, &MaterialType, &MeshType)>,
    mut layout_query: Query<&mut R>,
    mut last_items: Local<Vec<Entity>>,
) {
    for mut renderer in layout_query.iter_mut() {
        for source_entity in last_items.iter() {
            if source_item_query.get(*source_entity).is_err() {
                despawn_render_item(&mut renderer, source_entity, &mut commands);
            }
        }

        *last_items = source_item_query.iter().map(|item| item.0).collect();
    }
}

pub(crate) fn remove_moving_render_items<R: RenderMapApi + Component>(
    mut commands: Commands,
    source_item_query: Query<
        Entity,
        (
            With<HexPositionFractionalDelta>,
            With<MaterialType>,
            With<MeshType>,
        ),
    >,
    mut layout_query: Query<&mut R>,
) {
    for mut renderer in layout_query.iter_mut() {
        for source_entity in source_item_query.iter() {
            if source_item_query.get(source_entity).is_ok() {
                despawn_render_item(&mut renderer, &source_entity, &mut commands);
            }
        }
    }
}

pub(crate) fn set_camera_state<W: Component, const IS_ACTIVE: bool>(
    mut camera_query: Query<&mut Camera, With<W>>,
) {
    for mut camera in camera_query.iter_mut() {
        camera.is_active = IS_ACTIVE;
    }
}

// pub(crate) fn render_map<T: Bundle, R: CreateRenderBundle<T> + RenderMapApi + Component>(
//     mut commands: Commands,
//     mut render_map_event: EventReader<MapAddEvent>,
//     mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
// ) {
//     for e in render_map_event.read() {
//         for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
//             let mut spawned_hexes = Vec::new();

//             for (source_entity, map_bundle) in e.source_items.iter() {
//                 let pos_2d = layout.hex_to_pixel(&FractionalHexVector::from(&map_bundle.pos.0));
//                 let pos = Vec3::new(pos_2d.x, pos_2d.y, map_bundle.height.get_height().into());
//                 let render_bundle = renderer.create_render_bundle(
//                     &pos,
//                     &map_bundle.material_type,
//                     &map_bundle.mesh_type,
//                 );

//                 let render_entity = commands.spawn(render_bundle).id();
//                 let old_option = renderer.link_source_item(source_entity, &render_entity);

//                 if let Some(old_rendered_entity) = old_option {
//                     warn!("double render, replacing with new one");
//                     commands.entity(old_rendered_entity).despawn();
//                 }
//                 spawned_hexes.push(render_entity);
//             }

//             commands.entity(layout_entity).push_children(&spawned_hexes);
//         }
//     }
// }

// pub(crate) fn empty_map<R: RenderMapApi + Component>(
//     mut commands: Commands,
//     mut layout_query: Query<&mut R>,
//     map_data_query: Query<Entity, With<HexPosition>>,
// ) {
//     for mut renderer in layout_query.iter_mut() {
//         for source_entity in map_data_query.iter() {
//             if let Some(render_entity) = renderer.get_render_item(&source_entity) {
//                 commands.entity(*render_entity).remove_parent();
//                 commands.entity(*render_entity).despawn_recursive();
//             }
//             renderer.remove_render_item(&source_entity);
//         }
//     }
// }
// pub(crate) fn fill_map<T: Bundle, R: CreateRenderBundle<T> + RenderMapApi + Component>(
//     mut commands: Commands,
//     mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
//     map_data_query: Query<(Entity, &HexPosition, &Height, &MaterialType, &MeshType)>,
// ) {
//     for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
//         for (source_entity, pos, height, material_type, mesh_type) in map_data_query.iter() {
//             if renderer.get_render_item(&source_entity).is_some() {
//                 continue;
//             }

//             let pos_2d = layout.hex_to_pixel(&FractionalHexVector::from(&pos.0));
//             let pos = Vec3::new(pos_2d.x, pos_2d.y, height.get_height().into());
//             let render_bundle = renderer.create_render_bundle(&pos, material_type, mesh_type);
//             let render_entity = commands.spawn(render_bundle).id();
//             renderer.link_source_item(&source_entity, &render_entity);
//             commands.entity(layout_entity).add_child(render_entity);
//         }
//     }
// }
// pub(crate) fn despawn_map<R: RenderMapApi + Component>(
// 	mut commands: Commands,
// 	mut despawn_map_event: EventReader<MapSubEvent>,
// 	mut layout_query: Query<(Entity, &mut R)>,
// ) {
// 	for e in despawn_map_event.read() {
// 			for (layout_entity, mut render_map) in layout_query.iter_mut() {
// 					let mut children_to_remove: Vec<Entity> = Vec::new();
// 					for source_entity in e.source_items.iter() {
// 							if let Some(render_entity) = render_map.remove_render_item(source_entity) {
// 									commands.entity(render_entity).despawn_recursive();
// 									children_to_remove.push(render_entity);
// 							}
// 					}
// 					commands
// 							.entity(layout_entity)
// 							.remove_children(&children_to_remove);
// 			}
// 	}
// }

pub(crate) fn show_entity<E: Component>(mut commands: Commands, query: Query<Entity, With<E>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

pub(crate) fn move_rendered_items<R: RenderMapApi + Component>(
    mut transform_query: Query<&mut Transform>,
    moveable_query: Query<(Entity, &HexPositionFractionalDelta, &Height)>,
    layout_query: Query<(&HexLayout, &R)>,
) {
    for (source_entity, delta_pos, height) in moveable_query.iter() {
        if delta_pos.0.length() <= 0.0 {
            debug!("Skip updating character move");
            continue;
        }
        for (layout, renderer) in layout_query.iter() {
            let render_entity_option = renderer.get_render_item(&source_entity);
            if let Some(render_entity) = render_entity_option {
                if let Ok(mut transform) = transform_query.get_mut(*render_entity) {
                    let delta = layout.hex_to_pixel(&delta_pos.0);
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                    transform.translation.z = height.0.into();
                }
            } else {
                error!("Could not get character render entity");
            }
        }
    }
}

pub(crate) fn camera_update<R: RenderMapApi + Component>(
    mut camera_query: Query<(
        &Camera,
        &mut Transform,
        Option<&CameraOffset>,
        Option<&CameraRotation>,
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

            debug!("Handling camera at target {}", target_pos);

            for (camera, mut camera_transform, offset_option, rotation_option) in
                camera_query.iter_mut()
            {
                if !camera.is_active {
                    continue;
                }

                camera_transform.translation.x = target_pos.x;
                camera_transform.translation.y = target_pos.y;
                camera_transform.translation.z = target_pos.z;

                if let Some(offset) = offset_option {
                    camera_transform.translation.x += offset.0;
                    camera_transform.translation.y += offset.1;
                    camera_transform.translation.z += offset.2;
                }

                if let Some(rotation) = rotation_option {
                    // camera_transform.rotation = Quat::from(rotation);
                    camera_transform.rotate_around(target_pos, Quat::from(rotation));
                    camera_transform.look_at(target_pos, Vec3::Y);
                }
            }
        }
    }
}

pub(crate) fn camera_look_around(
    mut camera_query: Query<(&Camera, &mut CameraRotation)>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for e in motion_evr.read() {
        let time_delta = time.delta_seconds();
        for (camera, mut camera_rotation) in camera_query.iter_mut() {
            if camera.is_active {
                camera_rotation.0 += e.delta.x / 3.0 * time_delta;
                camera_rotation.1 += e.delta.y / 3.0 * time_delta;
            }
        }
    }
}

pub(crate) fn camera_zoom(
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

fn despawn_render_item<R: RenderMapApi + Component>(
    renderer: &mut Mut<R>,
    source_entity: &Entity,
    commands: &mut Commands,
) {
    if let Some(render_entity) = renderer.remove_render_item(source_entity) {
        commands.entity(render_entity).remove_parent();
        commands.entity(render_entity).despawn_recursive();
        debug!(
            "Despawned of source {:?} -> {:?}",
            source_entity, render_entity
        );
    } else {
        warn!("Could not clean render item {:?}", source_entity);
    }
}

fn spawn_render_item<T: Bundle, R: CreateRenderBundle<T> + RenderMapApi + Component>(
    commands: &mut Commands,
    mut renderer: Mut<R>,
    pos: Vec3,
    material_type: &MaterialType,
    mesh_type: &MeshType,
    source_entity: Entity,
    layout_entity: Entity,
) {
    let render_entity = commands
        .spawn(renderer.create_render_bundle(&pos, material_type, mesh_type))
        .id();

    debug!(
        "Spawning map item {:?} {:?} for source {:?} -> {:?}",
        mesh_type, material_type, source_entity, render_entity
    );

    renderer.link_source_item(&source_entity, &render_entity);

    commands.entity(layout_entity).add_child(render_entity);
}
