pub mod hex_map_item;
pub mod layout_orientation;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

use self::{
    hex_map_item::{
        hex_vector::{FractionalHexVector, HexVector},
        HexMapItem,
    },
    layout_orientation::HexLayoutOrientation,
};

pub struct HexLayout {
    pub orientation: HexLayoutOrientation,
    pub size: Vec2,
    pub origin: Vec2,
}

pub fn hex_to_pixel(layout: &HexLayout, h: &HexVector) -> Vec2 {
    let matrix = &layout.orientation;

    let x = (matrix.f0 * h.0 as f32 + matrix.f1 * h.1 as f32) * layout.size.x;
    let y = (matrix.f2 * h.0 as f32 + matrix.f3 * h.1 as f32) * layout.size.y;
    println!("{:?} -> x:{} y:{}", h, x, y);
    Vec2::from_array([x, y])
}

// pub fn pixel_to_hex(layout: &mut HexLayout, point: Vec2) -> HexVector {
//     let matrix: Mat2 = layout.orientation.pixel_to_hex();
//     let vec = matrix.mul_vec2(point - layout.origin) / layout.size;

//     FractionalHexVector::from(vec).into()
// }

impl HexLayout {
    fn ring_from(hex: &HexVector, radius: i16, f: fn(&HexVector)) {}
    fn spiral_from(hex: &HexVector, radius: i16, f: fn(&HexVector)) {
        f(hex);
        for i in 1..radius + 1 {
            Self::ring_from(hex, i, f)
        }
    }
}

pub trait PaintHex {
    fn paint(
        &self,
        layout: &HexLayout,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    );
}

impl PaintHex for HexMapItem {
    fn paint(
        &self,
        layout: &HexLayout,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let pos = hex_to_pixel(layout, &self.pos);
        let color = {
            let mut rng = rand::thread_rng();
            let y: f64 = rng.gen(); // generates a float between 0 and 1
            if y < 0.2 {
                Color::CRIMSON
            } else if y < 0.4 {
                Color::ALICE_BLUE
            } else if y < 0.6 {
                Color::SEA_GREEN
            } else if y < 0.8 {
                Color::GOLD
            } else {
                Color::GRAY
            }
        };

        let mut transform = Transform::from_xyz(pos.x, pos.y, 0.0);
        transform.rotate_z(layout.orientation.starting_angle.into());

        let bundle = MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(RegularPolygon::new(layout.size.x, 6))),
            material: materials.add(color),
            transform,
            ..Default::default()
        };
        let text_style: TextStyle = TextStyle {
            font_size: 16.0,
            color: Color::BLACK,
            ..Default::default()
        };
        let debug_bundles = vec![
            Text2dBundle {
                text: Text::from_section(format!("q: {}", self.pos.0), text_style.clone()),
                transform: Transform::from_xyz(pos.x, pos.y + 8.0, 1.0),
                ..Default::default()
            },
            Text2dBundle {
                text: Text::from_section(format!("r: {}", self.pos.1), text_style.clone()),
                transform: Transform::from_xyz(pos.x, pos.y - 8.0, 1.0),
                ..Default::default()
            },
        ];

        commands.spawn(bundle);
        commands.spawn(debug_bundles[0].clone());
        commands.spawn(debug_bundles[1].clone());
    }
}
