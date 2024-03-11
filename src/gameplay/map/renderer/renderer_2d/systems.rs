use crate::gameplay::{
    map::{
        renderer::{
            components::{RenderGroup, RenderGroupItem},
            events::RenderCharacterEvent,
        },
        spawner::{MapAddEvent, MapSubEvent},
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::{FractionalHexVector, HexVector},
        },
    },
    player::{components::HexPosition, events::CharacterMovedEvent},
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use super::resources::*;

const RENDER_GROUP: RenderGroup = RenderGroup::PreviewMap2D;

pub(crate) fn render_map(
    mut commands: Commands,
    mut render_map_event: EventReader<MapAddEvent>,
    mut render_map: ResMut<SourceToRenderStore2d>,
    layout_query: Query<(Entity, &HexLayout, &RenderGroup)>,
    meshes_map: Res<MeshesStore2d>,
    materials_map: Res<MaterialStore2d>,
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
            for source_entity in e.source_items.iter() {
                match map_data_query.get(*source_entity) {
                    Ok((hex_pos, biome, height)) => {
                        let pos = &hex_pos.0;
                        let transform = get_hex_transform(layout, pos);
                        let material = get_hex_material(&materials_map, height, biome);
                        let mesh = get_hex_mesh(&meshes_map);

                        let render_bundle = MaterialMesh2dBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            transform,
                            ..Default::default()
                        };

                        let rendered_hex_id = commands.spawn(render_bundle).id();
                        spawned_hexes.push(rendered_hex_id);
                        render_map.0.insert(source_entity.index(), rendered_hex_id);
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
    mut render_map: ResMut<SourceToRenderStore2d>,
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
            for source_entity in e.source_items.iter() {
                let index = &source_entity.index();
                if let Some(render_entity) = render_map.0.get(index) {
                    commands.entity(*render_entity).despawn_recursive();
                    children_to_remove.push(*render_entity);
                    render_map.0.remove(index);
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
pub(crate) fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), RenderGroupItem::PreviewMap2D));
}

pub(crate) fn despawn_camera(mut commands: Commands, camera_query: Query<Entity, With<Camera2d>>) {
    for c in camera_query.iter() {
        commands.entity(c).despawn_recursive();
    }
}

fn get_hex_mesh(meshes_map: &Res<MeshesStore2d>) -> Mesh2dHandle {
    Mesh2dHandle(
        meshes_map
            .0
            .get(&MeshKey2d::Hex)
            .expect("Could not get hex mesh")
            .clone(),
    )
}

pub(crate) fn render_character(
    mut commands: Commands,
    mut event: EventReader<RenderCharacterEvent>,
    mut render_map: ResMut<SourceToRenderStore2d>,
    layout_query: Query<(Entity, &HexLayout, &RenderGroup)>,
    mesh_map: Res<MeshesStore2d>,
    materials_map: Res<MaterialStore2d>,
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
                .get(&MeshKey2d::Character)
                .expect("Player mesh not found");

            let material_handle = materials_map
                .0
                .get(&e.material_key)
                .unwrap_or_else(|| panic!("could not get {} material", e.material_key));

            let render_entity = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handle.clone()),
                    material: material_handle.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, 2.0),
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

pub(crate) fn move_rendered_character(
    mut event: EventReader<CharacterMovedEvent>,
    mut transform_query: Query<&mut Transform>,
    render_map: ResMut<SourceToRenderStore2d>,
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

fn get_hex_transform(layout: &HexLayout, hex: &HexVector) -> Transform {
    let pos = layout.hex_to_pixel(&FractionalHexVector::from(hex));

    Transform::from_xyz(pos.x, pos.y, 0.0)
}

fn get_hex_material(
    materials_map: &Res<MaterialStore2d>,
    height: &Height,
    biome: &Biome,
) -> Handle<ColorMaterial> {
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
