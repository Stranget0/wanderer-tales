use crate::gameplay::{
    map::{
        renderer::{components::RenderGroup, events::RenderCharacterEvent},
        spawner::{MapAddEvent, MapSubEvent},
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::{FractionalHexVector, HexVector},
        },
    },
    player::{
        components::{HexPosition, WSADSteerable},
        events::CharacterMovedEvent,
    },
};
use bevy::prelude::*;

use super::resources::{MaterialStore3d, MeshKey3d, MeshesStore3d, SourceToRenderStore3d};

const RENDER_GROUP: RenderGroup = RenderGroup::Gameplay3D;

pub(crate) fn render_map(
    mut commands: Commands,
    mut render_map_event: EventReader<MapAddEvent>,
    mut render_map: ResMut<SourceToRenderStore3d>,
    meshes_map: Res<MeshesStore3d>,
    materials_map: Res<MaterialStore3d>,
    layout_query: Query<(Entity, &HexLayout, &RenderGroup)>,
    map_data_query: Query<(&HexPosition, &Biome, &Height)>,
) {
    for e in render_map_event.read() {
        if are_render_groups_irrelevant(&e.render_groups) {
            continue;
        }
        for (layout_entity, layout, layout_render_group) in layout_query.iter() {
            if is_render_group_irrelevant(layout_render_group) {
                continue;
            }

            let mut spawned_hexes = Vec::new();

            for hex_source in e.source_items.iter() {
                match map_data_query.get(*hex_source) {
                    Ok((pos, biome, height)) => {
                        let transform = get_hex_transform(layout, &pos.0, height);
                        let material = get_hex_material(&materials_map, height, biome);
                        let mesh = get_hex_mesh(&meshes_map);

                        let render_bundle = MaterialMeshBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            transform,
                            ..Default::default()
                        };

                        let rendered_hex_id = commands.spawn(render_bundle).id();
                        spawned_hexes.push(rendered_hex_id);
                        render_map.0.insert(hex_source.index(), rendered_hex_id);
                    }
                    Err(err) => {
                        error!("[renderer] entity get error: {}", err);
                        continue;
                    }
                };
            }
            commands.entity(layout_entity).push_children(&spawned_hexes);
        }
    }
}

pub(crate) fn despawn_map(
    mut commands: Commands,
    mut despawn_map_event: EventReader<MapSubEvent>,
    mut render_map: ResMut<SourceToRenderStore3d>,
    layout_query: Query<(Entity, &RenderGroup)>,
) {
    for e in despawn_map_event.read() {
        if are_render_groups_irrelevant(&e.render_groups) {
            continue;
        }
        for (layout_entity, layout_render_group) in layout_query.iter() {
            if is_render_group_irrelevant(layout_render_group) {
                continue;
            }

            let mut children_to_remove: Vec<Entity> = Vec::new();

            for hex_source in e.source_items.iter() {
                let index = hex_source.index();
                if let Some(render_entity) = render_map.0.get(&index) {
                    commands.entity(*render_entity).despawn_recursive();
                    children_to_remove.push(*render_entity);
                    render_map.0.remove(&index);
                }
            }
            commands
                .entity(layout_entity)
                .remove_children(&children_to_remove);
        }
    }
}

pub(crate) fn delete_render_groups(mut commands: Commands, groups: Query<(Entity, &RenderGroup)>) {
    for (map, render_group) in groups.iter() {
        if render_group == &RENDER_GROUP {
            commands.entity(map).despawn_recursive();
        }
    }
}

pub(crate) fn spawn_camera(
    mut commands: Commands,
    layout_query: Query<(&HexLayout, &RenderGroup)>,
) {
    for (layout, render_group) in layout_query.iter() {
        if render_group != &RENDER_GROUP {
            continue;
        };

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 1.5 * layout.size.x * 10.0, 300.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            WSADSteerable,
        ));
    }
}

pub(crate) fn despawn_camera(mut commands: Commands, camera_query: Query<Entity, With<Camera3d>>) {
    for c in camera_query.iter() {
        commands.entity(c).despawn_recursive();
    }
}

fn get_hex_mesh(meshes_map: &Res<MeshesStore3d>) -> Handle<Mesh> {
    meshes_map
        .0
        .get(&MeshKey3d::Hex)
        .expect("Could not get hex mesh")
        .clone()
}

pub(crate) fn render_character(
    mut commands: Commands,
    mut event: EventReader<RenderCharacterEvent>,
    mut render_map: ResMut<SourceToRenderStore3d>,
    layout_query: Query<(Entity, &HexLayout, &RenderGroup)>,
    mesh_map: Res<MeshesStore3d>,
    materials_map: Res<MaterialStore3d>,
) {
    for e in event.read() {
        if are_render_groups_irrelevant(&e.render_groups) {
            continue;
        }
        for (layout_entity, layout, layout_render_group) in layout_query.iter() {
            if is_render_group_irrelevant(layout_render_group) {
                continue;
            }
            let pos = layout.hex_to_pixel(&e.position.0);
            let mesh_handle = mesh_map
                .0
                .get(&MeshKey3d::Character)
                .expect("Character mesh not found");

            let material_handle = materials_map
                .0
                .get(&e.material_key)
                .expect("could not get material");

            let render_entity = commands
                .spawn(PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: material_handle.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, 256.0),
                    ..default()
                })
                .id();

            render_map
                .0
                .insert(e.character_entity.index(), render_entity);

            commands.entity(layout_entity).add_child(render_entity);
        }
    }
}

fn get_hex_transform(layout: &HexLayout, hex: &HexVector, height: &Height) -> Transform {
    let pos = layout.hex_to_pixel(&FractionalHexVector::from(hex));

    Transform::from_xyz(pos.x, pos.y, height.get_height().into())
}

pub(crate) fn move_rendered_character(
    mut event: EventReader<CharacterMovedEvent>,
    mut transform_query: Query<&mut Transform>,
    render_map: ResMut<SourceToRenderStore3d>,
    layout_query: Query<(&HexLayout, &RenderGroup)>,
) {
    for e in event.read() {
        for (layout, layout_render_group) in layout_query.iter() {
            if is_render_group_irrelevant(layout_render_group) {
                continue;
            }
            let render_entity_option = render_map.0.get(&e.character_source.index());
            if let Some(render_entity) = render_entity_option {
                if let Ok(mut transform) = transform_query.get_mut(*render_entity) {
                    let delta = layout.hex_to_pixel(&e.delta_pos.0);
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                }
            } else {
                error!("Could not get character render entity");
            }
        }
    }
}

fn get_hex_material(
    materials_map: &Res<MaterialStore3d>,
    height: &Height,
    biome: &Biome,
) -> Handle<StandardMaterial> {
    {
        let material_key = height.get_material();

        materials_map
            .0
            .get(&material_key)
            .unwrap_or_else(|| panic!("failed getting {material_key} material"))
            .clone()
    }
}

fn is_render_group_irrelevant(render_group: &RenderGroup) -> bool {
    render_group != &RENDER_GROUP
}

fn are_render_groups_irrelevant(render_groups: &[RenderGroup]) -> bool {
    !render_groups.contains(&RENDER_GROUP)
}
