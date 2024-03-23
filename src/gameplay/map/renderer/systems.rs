use crate::{
    gameplay::{
        map::utils::{
            hex_layout::{get_hex_corner_2d, HexLayout},
            hex_map_item::Height,
            hex_vector::FractionalHexVector,
        },
        player::components::{
            HexPosition, HexPositionFractional, HexPositionFractionalDelta, PlayerRoot, Rotation,
        },
    },
    utils::{to_3d_space, EULER_ROT, FORWARD, UP},
};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    utils::info,
};

use super::{
    components::{CameraOffset, CameraRotation, MaterialType, MeshType, SourceCameraFollow},
    renderers::{
        meshes::Hexagon3D,
        renderer_2d::Renderer2D,
        renderer_3d::Renderer3D,
        traits::{CreateRenderBundles, RenderMapApi},
    },
};

pub(crate) fn render_static_map_items<
    T: Bundle,
    R: CreateRenderBundles<T> + RenderMapApi + Component,
>(
    mut commands: Commands,
    render_type_query: Query<(
        Entity,
        &HexPosition,
        &Height,
        &Rotation,
        &MeshType,
        &MaterialType,
    )>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
) {
    for (source_entity, position, height, rotation, mesh_type, material_type) in
        render_type_query.iter()
    {
        for (layout_entity, layout, renderer) in layout_query.iter_mut() {
            if renderer.get_render_item(&source_entity).is_some() {
                continue;
            }
            let pos_2d = layout.hex_to_pixel(&FractionalHexVector::from(&position.0));
            let pos = UP * f32::from(height.0) + FORWARD * pos_2d.y + Vec3::X * pos_2d.x;
            let (render_bundle, render_children) =
                renderer.create_render_bundle(&pos, &rotation, material_type, mesh_type);

            spawn_render_item(
                &mut commands,
                renderer,
                render_bundle,
                source_entity,
                layout_entity,
                render_children,
            );
        }
    }
}

pub(crate) fn render_map_items<T: Bundle, R: CreateRenderBundles<T> + RenderMapApi + Component>(
    mut commands: Commands,
    render_type_query: Query<(
        Entity,
        &HexPositionFractional,
        &Height,
        &Rotation,
        &MeshType,
        &MaterialType,
    )>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
) {
    for (source_entity, position, height, rotation, mesh_type, material_type) in
        render_type_query.iter()
    {
        for (layout_entity, layout, renderer) in layout_query.iter_mut() {
            if renderer.get_render_item(&source_entity).is_some() {
                continue;
            }
            let pos_2d = layout.hex_to_pixel(&position.0);
            let pos = UP * f32::from(height.0) + FORWARD * pos_2d.y + Vec3::X * pos_2d.x;
            let (render_bundle, render_children) =
                renderer.create_render_bundle(&pos, rotation, material_type, mesh_type);

            spawn_render_item(
                &mut commands,
                renderer,
                render_bundle,
                source_entity,
                layout_entity,
                render_children,
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
                    let [x, y, z] = to_3d_space(
                        transform.translation.x + delta.x,
                        transform.translation.y + delta.y,
                        height.0.into(),
                    );
                    transform.translation.x = x;
                    transform.translation.y = y;
                    transform.translation.z = z;
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
                    let rotation_1 = Quat::from_euler(EULER_ROT, 0.0, 0.0, -rotation.0);
                    let rotation_2 = Quat::from_euler(EULER_ROT, -rotation.1, 0.0, 0.0);
                    camera_transform.translate_around(target_pos, rotation_1 * rotation_2);
                    camera_transform.look_at(target_pos, UP);
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
                let sensitivity = 20.0;
                camera_rotation.0 += (e.delta.x * time_delta * sensitivity).to_radians();
                camera_rotation.1 += (e.delta.y * time_delta * sensitivity).to_radians();
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

pub(crate) fn debug_heights_2d(
    mut commands: Commands,
    height_query: Query<(Entity, &Height, &MeshType, &Rotation), Without<PlayerRoot>>,
    player_query: Query<&Height, With<PlayerRoot>>,
    renderer: Query<(&HexLayout, &Renderer2D)>,
) {
    for (source_entity, h, mesh_type, rotation) in height_query.iter() {
        for (l, r) in renderer.iter() {
            let player_h = player_query.single();
            if let Some(render_entity) = r.get_render_item(&source_entity) {
                if let MeshType::HexMapTile(cycle) = mesh_type {
                    let relative_h = commands
                        .spawn(Text2dBundle {
                            text: Text::from_section(
                                format!("{}", h.0 as i16 - player_h.0 as i16),
                                TextStyle {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ),
                            transform: Transform::from_xyz(0.0, 0.0, 3.0),
                            ..default()
                        })
                        .id();

                    let ids =
                        Hexagon3D::get_top_vertices(l.orientation.starting_angle, l.size.x, &cycle)
                            .map(|[x, y, z]| {
                                let mut transform = Transform::from_xyz(x * 0.8, y * 0.8, 3.0);
                                transform.rotate(Quat::from_euler(
                                    EULER_ROT,
                                    -rotation.0.x,
                                    -rotation.0.y,
                                    -rotation.0.z,
                                ));
                                commands
                                    .spawn(Text2dBundle {
                                        text: Text::from_section(
                                            format!("{:.1}", z),
                                            TextStyle {
                                                font_size: 14.0,
                                                ..default()
                                            },
                                        ),
                                        transform,
                                        ..default()
                                    })
                                    .id()
                            });

                    commands
                        .entity(*render_entity)
                        .replace_children(&[[relative_h].to_vec(), ids.to_vec()].concat()[..]);
                }
            }
        }
    }
}

pub(crate) fn debug_heights_cycle_3d(
    mut commands: Commands,
    height_query: Query<(Entity, &MeshType)>,
    renderer_query: Query<(&HexLayout, &Renderer3D)>,
) {
    for (source_entity, mesh_type) in height_query.iter() {
        if let MeshType::HexMapTile(h_cycle) = mesh_type {
            for (layout, renderer) in renderer_query.iter() {
                if let Some(render_entity) = renderer.get_render_item(&source_entity) {
                    for [x, y, z] in Hexagon3D::get_top_vertices(
                        layout.orientation.starting_angle,
                        layout.size.x,
                        h_cycle,
                    )
                    .iter()
                    .take(1)
                    {
                        let entity = commands
                            .spawn(PbrBundle {
                                mesh: renderer.meshes_map.get(&MeshType::Debug).unwrap().clone(),
                                material: renderer
                                    .materials_map
                                    .get(&MaterialType::Debug)
                                    .unwrap()
                                    .clone(),
                                transform: Transform::from_xyz(*x, *y, *z),
                                ..default()
                            })
                            .id();

                        commands
                            .entity(*render_entity)
                            .replace_children(&[entity][..]);
                    }
                }
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

fn spawn_render_item<T: Bundle, R: CreateRenderBundles<T> + RenderMapApi + Component>(
    commands: &mut Commands,
    mut renderer: Mut<R>,
    bundle: T,
    source_entity: Entity,
    layout_entity: Entity,
    children: Option<Vec<T>>,
) -> Entity {
    let render_entity = commands.spawn(bundle).id();

    debug_spawn(&source_entity, &render_entity);

    renderer.link_source_item(&source_entity, &render_entity);

    commands.entity(layout_entity).add_child(render_entity);
    if let Some(bundles) = children {
        for b in bundles {
            let child = commands.spawn(b).id();
            commands.entity(render_entity).add_child(child);
        }
    }

    render_entity
}

fn debug_spawn(source_entity: &Entity, render_entity: &Entity) {
    debug!(
        "[Spawning map item {:?} -> {:?}]",
        source_entity, render_entity
    );
}
