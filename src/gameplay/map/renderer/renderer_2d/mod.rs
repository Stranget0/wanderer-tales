use self::resources::{MaterialStore, MeshKey, MeshesStore};
use super::events::RenderCharacter;
use crate::gameplay::map::{
    events::MapAddEvent,
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

pub mod resources;

pub fn render_map(
    mut commands: Commands,
    meshes_map: Res<MeshesStore>,
    materials_map: Res<MaterialStore>,
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

fn get_hex_mesh(meshes_map: &Res<MeshesStore>) -> Mesh2dHandle {
    Mesh2dHandle(
        meshes_map
            .0
            .get(&MeshKey::Hex)
            .expect("Could not get hex mesh")
            .clone(),
    )
}

pub fn render_point(
    mut commands: Commands,
    mut event: EventReader<RenderCharacter>,
    mesh_map: Res<MeshesStore>,
    materials_map: Res<MaterialStore>,
) {
    for e in event.read() {
        let mesh_handle = mesh_map
            .0
            .get(&MeshKey::Player)
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
    materials_map: &Res<MaterialStore>,
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
