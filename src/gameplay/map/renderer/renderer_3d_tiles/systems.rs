use crate::gameplay::map::{
    renderer::events::RenderCharacterEvent,
    spawner::MapAddEvent,
    utils::{
        hex_layout::HexLayout,
        hex_map_item::{Biome, Height},
        hex_vector::HexVector,
    },
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use super::resources::{MaterialStore3d, MeshKey3d, MeshesStore3d};

pub(crate) fn render_map(
    mut commands: Commands,
    meshes_map: Res<MeshesStore3d>,
    materials_map: Res<MaterialStore3d>,
    layout_query: Query<&HexLayout>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map_data_query: Query<(&HexVector, &Biome, &Height)>,
    mut render_map_event: EventReader<MapAddEvent>,
) {
    for event in render_map_event.read() {
        let layout = layout_query.single();
        for hex_entity in event.0.iter() {
            match map_data_query.get(*hex_entity) {
                Ok((pos, biome, height)) => {
                    let transform = get_hex_transform(layout, pos);
                    let material = get_hex_material(&materials_map, &mut materials, height, biome);
                    let mesh = get_hex_mesh(&meshes_map);

                    let render_bundle = MaterialMesh2dBundle {
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

pub(crate) fn free_map(
    mut commands: Commands,
    rendered_items_query: Query<Entity, With<Mesh2dHandle>>,
) {
    for item in rendered_items_query.iter() {
        commands
            .entity(item)
            .remove::<MaterialMesh2dBundle<ColorMaterial>>();
    }
}

pub(crate) fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default());
}

pub(crate) fn despawn_camera(mut commands: Commands, camera_query: Query<Entity, With<Camera3d>>) {
    for c in camera_query.iter() {
        commands.entity(c).despawn_recursive();
    }
}

fn get_hex_mesh(meshes_map: &Res<MeshesStore3d>) -> Mesh2dHandle {
    Mesh2dHandle(
        meshes_map
            .0
            .get(&MeshKey3d::Hex)
            .expect("Could not get hex mesh")
            .clone(),
    )
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
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle.clone()),
                material: material_handle.clone(),
                ..default()
            })
            .id();

        commands.entity(e.entity).add_child(child);
    }
}

fn get_hex_transform(layout: &HexLayout, hex: &HexVector) -> Transform {
    let pos = layout.hex_to_pixel(hex);

    Transform::from_xyz(pos.x, pos.y, 0.0)
}

fn get_hex_material(
    materials_map: &Res<MaterialStore3d>,
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
