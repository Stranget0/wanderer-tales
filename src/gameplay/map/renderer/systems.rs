use crate::gameplay::components::*;
use crate::gameplay::map::utils::*;
use crate::utils::*;
use bevy::{ecs::query::QueryFilter, prelude::*};

use super::{
    components::*,
    renderers::traits::{CreateRenderBundles, RenderMapApi},
};

pub(crate) fn render_map_items<
    T: Bundle,
    M: Asset,
    R: CreateRenderBundles<T, M> + RenderMapApi + Component,
    F: QueryFilter,
>(
    mut commands: Commands,
    render_type_query: Query<
        (
            Entity,
            AnyOf<(&HexPosition, &HexPositionFractional)>,
            &Height,
            &Rotation,
            &MeshType,
            &MaterialType,
        ),
        F,
    >,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<M>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    for (source_entity, position, height, rotation, mesh_type, material_type) in
        render_type_query.iter()
    {
        for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
            if let Some(render_entity) = renderer.get_render_item(&source_entity) {
                debug!("changing already rendered item by despawning");
                commands.entity(*render_entity).despawn();
                renderer.remove_render_item(&source_entity);
            }
            let pos = match position.0 {
                Some(pos) => FractionalHexVector::from(&pos.0),
                None => position.1.unwrap().0,
            };
            let pos_2d = layout.hex_to_pixel(&pos);
            let pos = UP * f32::from(height.0) + FORWARD * pos_2d.y + Vec3::X * pos_2d.x;
            let render_bundle = renderer.create_render_bundle(
                &pos,
                rotation,
                material_type,
                mesh_type,
                layout,
                &mut materials,
                &mut images,
                &mut meshes,
                &asset_server,
            );

            spawn_render_item(
                &mut commands,
                renderer,
                render_bundle,
                source_entity,
                layout_entity,
            );
        }
    }
}

pub(crate) fn clean_render_items<R: RenderMapApi + Component>(
    mut commands: Commands,
    mut removed_source_items: RemovedComponents<MeshType>,
    mut layout_query: Query<&mut R>,
) {
    for source_entity in removed_source_items.read() {
        for mut renderer in layout_query.iter_mut() {
            despawn_render_item(&mut renderer, &source_entity, &mut commands);
        }
    }
}

pub(crate) fn clear_unrelevant_render_items<R: RenderMapApi + Component>(
    mut commands: Commands,
    source_items: Query<Entity, With<MeshType>>,
    mut layout_query: Query<&mut R>,
) {
    for source_entity in source_items.iter() {
        for mut renderer in layout_query.iter_mut() {
            despawn_render_item(&mut renderer, &source_entity, &mut commands);
        }
    }
}

pub(crate) fn clear_all_render_items<R: RenderMapApi + Component>(
    mut commands: Commands,
    source_items: Query<Entity, With<MeshType>>,
    mut layout_query: Query<&mut R>,
) {
    for source_entity in source_items.iter() {
        for mut renderer in layout_query.iter_mut() {
            despawn_render_item(&mut renderer, &source_entity, &mut commands);
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

pub(crate) fn show_entity<E: Component>(mut commands: Commands, query: Query<Entity, With<E>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

pub(crate) fn move_rendered_items<R: RenderMapApi + Component>(
    mut transform_query: Query<&mut Transform>,
    moveable_query: Query<
        (Entity, &HexPositionFractional, &Height),
        Or<(
            Changed<HexPositionFractional>,
            Added<HexPositionFractional>,
            Changed<Height>,
            Added<Height>,
        )>,
    >,
    layout_query: Query<(&HexLayout, &R)>,
) {
    for (source_entity, pos, height) in moveable_query.iter() {
        for (layout, renderer) in layout_query.iter() {
            let render_entity_option = renderer.get_render_item(&source_entity);
            if let Some(mut transform) = render_entity_option
                .and_then(|render_entity| transform_query.get_mut(*render_entity).ok())
            {
                let pos = layout.hex_to_pixel(&pos.0);
                let [x, y, z] = to_3d_space(pos.x, pos.y, height.0.into());
                transform.translation.x = x;
                transform.translation.y = y;
                transform.translation.z = z;
            } else {
                debug!("Could not move render item");
            }
        }
    }
}

pub(crate) fn rotate_rendered_items<R: RenderMapApi + Component>(
    mut transform_query: Query<&mut Transform>,
    rotatable_query: Query<(Entity, &Rotation), Or<(Changed<Rotation>, Added<Rotation>)>>,
    layout_query: Query<&R>,
) {
    for (source_entity, rotation) in rotatable_query.iter() {
        for renderer in layout_query.iter() {
            let render_entity_option = renderer.get_render_item(&source_entity);
            if let Some(render_entity) = render_entity_option {
                if let Ok(mut transform) = transform_query.get_mut(*render_entity) {
                    transform.rotation = rotation.0;
                }
            } else {
                debug!("Could not rotate render item");
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
        debug!(
            "Could not clean render item {:?} [count {}]",
            source_entity,
            renderer.count()
        );
    }
}

fn spawn_render_item<
    T: Bundle,
    M: Asset,
    R: CreateRenderBundles<T, M> + RenderMapApi + Component,
>(
    commands: &mut Commands,
    mut renderer: Mut<R>,
    bundle: T,
    source_entity: Entity,
    layout_entity: Entity,
) -> Entity {
    let render_entity = commands.spawn(bundle).id();

    debug!(
        "[Spawning map item {:?} -> {:?}]",
        source_entity, render_entity
    );
    renderer.link_source_item(&source_entity, &render_entity);

    commands.entity(layout_entity).add_child(render_entity);

    render_entity
}

// #region old debug
// pub(crate) fn debug_heights_2d(
//     mut commands: Commands,
//     hex_query: Query<(Entity, &Height, &MeshType, &Rotation, &HexPosition), Without<PlayerRoot>>,
//     renderer: Query<(&HexLayout, &Renderer2D)>,
//     mut logged_entities: Local<Vec<Entity>>,
// ) {
//     for (source_entity, h, mesh_type, rotation, position) in hex_query.iter() {
//         // let player_pos = HexVector::from(&player_query.single().0 .0);
//         if logged_entities.contains(&source_entity) {
//             continue;
//         }
//         for (l, r) in renderer.iter() {
//             if let Some(render_entity) = r.get_render_item(&source_entity) {
//                 if let MeshType::HexMapTile(cycle) = mesh_type {
//                     logged_entities.push(source_entity);
//                     let heights = commands
//                         .spawn(Text2dBundle {
//                             text: Text::from_section(
//                                 format!("{}", h.0),
//                                 TextStyle {
//                                     font_size: 24.0,
//                                     ..default()
//                                 },
//                             ),
//                             transform: Transform::from_xyz(0.0, 0.0, 3.0),
//                             ..default()
//                         })
//                         .id();

//                     let corner_heights =
//                         Hexagon3D::get_top_vertices(l.orientation.starting_angle, l.size.x, cycle)
//                             .map(|[x, y, z]| {
//                                 let mut transform = Transform::from_xyz(x * 0.8, y * 0.8, 3.0);
//                                 transform.rotate(Quat::from_euler(
//                                     EULER_ROT,
//                                     -rotation.0.x,
//                                     -rotation.0.y,
//                                     -rotation.0.z,
//                                 ));
//                                 commands
//                                     .spawn(Text2dBundle {
//                                         text: Text::from_section(
//                                             format!("{:.1}", z + h.0 as f32),
//                                             TextStyle {
//                                                 font_size: 14.0,
//                                                 ..default()
//                                             },
//                                         ),
//                                         transform,
//                                         ..default()
//                                     })
//                                     .id()
//                             })
//                             .to_vec();

//                     // let sibling_order = (0..6)
//                     //     .map(|i| {
//                     //         let pos = l.hex_to_pixel(&FractionalHexVector::from(
//                     //             position.0.get_sibling(i),
//                     //         ));
//                     //         commands
//                     //             .spawn(Text2dBundle {
//                     //                 text: Text::from_section(
//                     //                     format!("{} \n {} {}", i, pos.x, pos.y),
//                     //                     TextStyle {
//                     //                         font_size: 24.0,
//                     //                         ..default()
//                     //                     },
//                     //                 ),
//                     //                 transform: Transform::from_xyz(pos.x, pos.y, 3.0),
//                     //                 ..default()
//                     //             })
//                     //             .id()
//                     //     })
//                     //     .collect_vec();

//                     let corner_order = (0..6)
//                         .map(|i| {
//                             let [x, y] =
//                                 get_hex_corner_2d(i, l.orientation.starting_angle, l.size.x);
//                             commands
//                                 .spawn(Text2dBundle {
//                                     text: Text::from_section(
//                                         format!("{}", i),
//                                         TextStyle {
//                                             font_size: 16.0,
//                                             ..default()
//                                         },
//                                     ),
//                                     transform: Transform::from_xyz(x * 0.9, y * 0.9, 3.0),
//                                     ..default()
//                                 })
//                                 .id()
//                         })
//                         .collect_vec();

//                     commands.entity(*render_entity).replace_children(
//                         &[vec![heights], corner_heights, corner_order].concat()[..],
//                     );
//                 }
//             }
//         }
//     }
// }

// pub(crate) fn debug_heights_cycle_3d(
//     mut commands: Commands,
//     height_query: Query<(Entity, &MeshType)>,
//     renderer_query: Query<(&HexLayout, &Renderer3D)>,
// ) {
//     for (source_entity, mesh_type) in height_query.iter() {
//         if let MeshType::HexMapTile(h_cycle) = mesh_type {
//             for (layout, renderer) in renderer_query.iter() {
//                 if let Some(render_entity) = renderer.get_render_item(&source_entity) {
//                     for [x, y, z] in Hexagon3D::get_top_vertices(
//                         layout.orientation.starting_angle,
//                         layout.size.x,
//                         h_cycle,
//                     )
//                     .iter()
//                     .take(1)
//                     {
//                         let entity = commands
//                             .spawn(PbrBundle {
//                                 mesh: renderer.meshes_map.get(&MeshType::Debug).unwrap().clone(),
//                                 material: renderer
//                                     .materials_map
//                                     .get(&MaterialType::Debug)
//                                     .unwrap()
//                                     .clone(),
//                                 transform: Transform::from_xyz(*x, *y, *z),
//                                 ..default()
//                             })
//                             .id();

//                         commands
//                             .entity(*render_entity)
//                             .replace_children(&[entity][..]);
//                     }
//                 }
//             }
//         }
//     }
// }
// #endregion
