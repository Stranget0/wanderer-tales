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

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn render_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    layout_q: Query<&HexLayout>,
    query: Query<&HexMapItemBundle>,
) {
    let layout = layout_q.single();

    for hex in query.iter() {
        let bundle = create_hex_bundle(
            layout,
            hex,
            Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
            &mut materials,
        );
        commands.spawn(bundle);
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

// pub fn pixel_to_hex(layout: &mut HexLayout, point: Vec2) -> HexVector {
//     let matrix: Mat2 = layout.orientation.pixel_to_hex();
//     let vec = matrix.mul_vec2(point - layout.origin) / layout.size;

//     FractionalHexVector::from(vec).into()
// }

// pub trait PaintHex {
//     fn paint(
//         &self,
//         layout: &HexLayout,
//         commands: &mut Commands,
//         meshes: &mut ResMut<Assets<Mesh>>,
//         materials: &mut ResMut<Assets<ColorMaterial>>,
//     );
// }

// impl PaintHex for HexMapItem {
//     fn paint(
//         &self,
//         layout: &HexLayout,
//         commands: &mut Commands,
//         meshes: &mut ResMut<Assets<Mesh>>,
//         materials: &mut ResMut<Assets<ColorMaterial>>,
//     ) {
//         let pos = hex_to_pixel(layout, &self.pos);
//         let color = {
//             let mut rng = rand::thread_rng();
//             let y: f64 = rng.gen(); // generates a float between 0 and 1
//             if y < 0.2 {
//                 Color::CRIMSON
//             } else if y < 0.4 {
//                 Color::ALICE_BLUE
//             } else if y < 0.6 {
//                 Color::SEA_GREEN
//             } else if y < 0.8 {
//                 Color::GOLD
//             } else {
//                 Color::GRAY
//             }
//         };

//         let mut transform = Transform::from_xyz(pos.x, pos.y, 0.0);
//         transform.rotate_z(layout.orientation.starting_angle);

//         let bundle = MaterialMesh2dBundle {
//             mesh: Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
//             material: materials.add(color),
//             transform,
//             ..Default::default()
//         };
//         let text_style: TextStyle = TextStyle {
//             font_size: 16.0,
//             color: Color::BLACK,
//             ..Default::default()
//         };
//         let debug_bundles = vec![
//             Text2dBundle {
//                 text: Text::from_section(format!("q: {}", self.pos.0), text_style.clone()),
//                 transform: Transform::from_xyz(pos.x, pos.y + 8.0, 1.0),
//                 ..Default::default()
//             },
//             Text2dBundle {
//                 text: Text::from_section(format!("r: {}", self.pos.1), text_style.clone()),
//                 transform: Transform::from_xyz(pos.x, pos.y - 8.0, 1.0),
//                 ..Default::default()
//             },
//         ];

//         commands.spawn(bundle);
//         commands.spawn(debug_bundles[0].clone());
//         commands.spawn(debug_bundles[1].clone());
//     }
// }
