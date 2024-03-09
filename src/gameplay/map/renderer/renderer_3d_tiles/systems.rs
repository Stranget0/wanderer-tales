use crate::gameplay::{
    map::{
        components::MapContent,
        renderer::events::RenderCharacterEvent,
        spawner::MapAddEvent,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::HexVector,
        },
    },
    player::components::WSADSteerable,
};
use bevy::prelude::*;

use super::resources::{MaterialStore3d, MeshKey3d, MeshesStore3d};

pub(crate) fn render_map(
    mut commands: Commands,
    meshes_map: Res<MeshesStore3d>,
    materials_map: Res<MaterialStore3d>,
    layout: Res<HexLayout>,
    map_data_query: Query<(&HexVector, &Biome, &Height)>,
    mut render_map_event: EventReader<MapAddEvent>,
) {
    for event in render_map_event.read() {
        for hex_entity in event.0.iter() {
            match map_data_query.get(*hex_entity) {
                Ok((pos, biome, height)) => {
                    let transform = get_hex_transform(&layout, pos);
                    let material = get_hex_material(&materials_map, height, biome);
                    let mesh = get_hex_mesh(&meshes_map);

                    let render_bundle = MaterialMeshBundle {
                        mesh,
                        material,
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

pub(crate) fn spawn_camera(mut commands: Commands, layout: Res<HexLayout>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(
                0.,
                1.5 * layout.size.x * 10.0,
                6. * layout.size.x * 10.0,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        WSADSteerable,
    ));
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
    mesh_map: Res<MeshesStore3d>,
    materials_map: Res<MaterialStore3d>,
) {
    for e in event.read() {
        let mesh_handle = mesh_map
            .0
            .get(&MeshKey3d::Character)
            .expect("Player mesh not found");

        let material_handle = materials_map
            .0
            .get(&e.material_key)
            .expect("could not get material");

        let child = commands
            .spawn(PbrBundle {
                mesh: mesh_handle.clone(),
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
