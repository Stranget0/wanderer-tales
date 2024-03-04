use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::gameplay::{
    map::{
        events::MapAddEvent,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::HexVector,
        },
    },
    theme::constants::COLORS,
};

use super::events::RenderPointEvent;

pub fn render_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    layout_query: Query<&HexLayout>,
    map_data_query: Query<(&HexVector, &Biome, &Height)>,
    mut render_map_event: EventReader<MapAddEvent>,
) {
    for event in render_map_event.read() {
        let layout = layout_query.single();
        for hex_entity in event.0.iter() {
            match map_data_query.get(*hex_entity) {
                Ok((pos, biome, height)) => {
                    let hex_bundle = create_hex_bundle(
                        layout,
                        height,
                        biome,
                        pos,
                        Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
                        &mut materials,
                    );
                    commands.entity(*hex_entity).insert(hex_bundle);
                }
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };
        }
    }
}

pub fn render_hex() {}
pub fn render_point(
    mut commands: Commands,
    mut event: EventReader<RenderPointEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for e in event.read() {
        let child = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: e.size })),
                material: materials.add(e.color),
                ..default()
            })
            .id();

        commands.entity(e.parent).add_child(child);
    }
}

fn create_hex_bundle(
    layout: &HexLayout,
    height: &Height,
    biome: &Biome,
    hex_vector: &HexVector,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> MaterialMesh2dBundle<ColorMaterial> {
    let transform = get_hex_transform(layout, hex_vector);
    let base_color = get_hex_material(height, biome);

    MaterialMesh2dBundle {
        mesh,
        material: materials.add(base_color),
        transform,
        ..Default::default()
    }
}

fn get_hex_transform(layout: &HexLayout, hex: &HexVector) -> Transform {
    let pos = layout.hex_to_pixel(hex);

    Transform::from_xyz(pos.x, pos.y, 0.0)
}

fn get_hex_material(height: &Height, biome: &Biome) -> Color {
    {
        match height.0 {
            height if height < 100 => COLORS.blue.l900,
            height if height > 200 => COLORS.gray.l900,
            _height => match biome {
                Biome::Grass => COLORS.green.l400,
                Biome::Forest => COLORS.green.l400,
            },
        }
    }
}
