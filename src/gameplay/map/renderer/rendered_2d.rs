use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::gameplay::{
    map::{
        events::MapAddEvent,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height, HexMapItemBundle},
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
    layout_query: Query<(Entity, &HexLayout)>,
    map_data_query: Query<(&HexVector, &Biome, &Height)>,
    mut render_event: EventReader<MapAddEvent>,
) {
    for event in render_event.read() {
        match layout_query.get_single() {
            Ok((layout_entity, layout)) => {
                for hex_entity in event.0.iter() {
                    match map_data_query.get(*hex_entity) {
                        Ok((pos, biome, height)) => {
                            let hex_bundle = create_hex_bundle(
                                layout,
                                &HexMapItemBundle {
                                    biome: *biome,
                                    height: *height,
                                    pos: pos.clone(),
                                },
                                Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
                                &mut materials,
                            );
                            let hex_entity = commands.spawn(hex_bundle).id();

                            commands.entity(layout_entity).add_child(hex_entity);
                        }
                        Err(err) => {
                            error!("{}", err);
                            continue;
                        }
                    };
                }
            }
            Err(err) => error!("Error getting layout: {}", err),
        }
    }
}

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
    hex: &HexMapItemBundle,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> MaterialMesh2dBundle<ColorMaterial> {
    let transform = get_hex_transform(layout, hex);
    let base_color = get_hex_material(hex);

    MaterialMesh2dBundle {
        mesh,
        material: materials.add(base_color),
        transform,
        ..Default::default()
    }
}

fn get_hex_transform(layout: &HexLayout, hex: &HexMapItemBundle) -> Transform {
    let pos = layout.hex_to_pixel(&hex.pos);

    Transform::from_xyz(pos.x, pos.y, 0.0)
}

fn get_hex_material(hex: &HexMapItemBundle) -> Color {
    {
        match hex.height.0 {
            height if height < 100 => COLORS.blue.l900,
            height if height > 200 => COLORS.gray.l900,
            _height => match hex.biome {
                Biome::Grass => COLORS.green.l400,
                Biome::Forest => COLORS.green.l400,
            },
        }
    }
}
