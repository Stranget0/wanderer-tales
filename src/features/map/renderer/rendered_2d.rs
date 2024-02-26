use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::features::{
    map::{
        hex_layout::HexLayout,
        hex_map_item::{Biome, HexMapItemBundle},
    },
    theme::constants::COLORS,
};

pub fn render_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    layout_entity_q: Query<Entity, With<HexLayout>>,
    layout_q: Query<&HexLayout>,
    query: Query<&HexMapItemBundle>,
) {
    let layout = layout_q.single();
    let layout_entity = layout_entity_q.single();

    for hex in query.iter() {
        let hex_bundle = create_hex_bundle(
            layout,
            hex,
            Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
            &mut materials,
        );
        let hex_entity = commands.spawn(hex_bundle).id();

        commands.entity(layout_entity).add_child(hex_entity);
    }
}

fn create_hex_bundle(
    layout: &HexLayout,
    hex: &HexMapItemBundle,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
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
