use crate::gameplay::{
    map::{
        components::MapContent,
        renderer::{components::MaterialKey, events::RenderCharacterEvent},
        spawner::MapAddEvent,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::{iterators::HexCorners, HexVector},
        },
    },
    player::components::WSADSteerable,
};
use bevy::{
    math::vec3,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use super::resources::*;

pub(crate) fn render_map(
    mut commands: Commands,
    meshes_map: Res<MeshesStore2d>,
    materials_map: Res<MaterialStore2d>,
    layout: Res<HexLayout>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map_data_query: Query<(&HexVector, &Biome, &Height)>,
    mut render_map_event: EventReader<MapAddEvent>,
) {
    for event in render_map_event.read() {
        for hex_entity in event.0.iter() {
            match map_data_query.get(*hex_entity) {
                Ok((pos, biome, height)) => {
                    let transform = get_hex_transform(&layout, pos);
                    let material = get_hex_material(&materials_map, &mut materials, height, biome);
                    let mesh = get_hex_mesh(&meshes_map);

                    let render_bundle = MaterialMesh2dBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform,
                        ..Default::default()
                    };

                    commands.entity(*hex_entity).insert(render_bundle);
                }
                Err(err) => {
                    error!("[renderer] entity get error: {}", err);
                    continue;
                }
            };
        }
    }
}

pub(crate) fn delete_maps(mut commands: Commands, maps_query: Query<Entity, With<MapContent>>) {
    for map in maps_query.iter() {
        commands.entity(map).despawn_recursive();
    }
}
pub(crate) fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), WSADSteerable));
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
            .get(&MeshKey::Hex)
            .expect("Could not get hex mesh")
            .clone(),
    )
}

pub(crate) fn render_character(
    mut commands: Commands,
    mut event: EventReader<RenderCharacterEvent>,
    mesh_map: Res<MeshesStore2d>,
    materials_map: Res<MaterialStore2d>,
) {
    for e in event.read() {
        let mesh_handle = mesh_map
            .0
            .get(&MeshKey::Character)
            .expect("Player mesh not found");

        let material_handle = materials_map
            .0
            .get(&e.material_key)
            .unwrap_or_else(|| panic!("could not get {} material", e.material_key));

        let child = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle.clone()),
                material: material_handle.clone(),
                ..default()
            })
            .id();

        commands.entity(e.parent).add_child(child);
    }
}

fn get_hex_transform(layout: &HexLayout, hex: &HexVector) -> Transform {
    let pos = layout.hex_to_pixel(hex);

    Transform::from_xyz(pos.x, pos.y, 0.0)
}

fn get_hex_material(
    materials_map: &Res<MaterialStore2d>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    height: &Height,
    biome: &Biome,
) -> Handle<ColorMaterial> {
    {
        let material_key = height.get_material();
        let handle = materials_map
            .0
            .get(&material_key)
            .expect("failed getting mountain material");

        let color = materials.get(handle).unwrap().color;

        let mut l: f32 = f32::from(height.get_height());
        l = l.floor() / 255.;
        let modified_color = color.with_l(l);

        materials.add(modified_color)
    }
}
