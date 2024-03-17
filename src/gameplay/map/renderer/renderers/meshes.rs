use crate::gameplay::map::utils::hex_layout::get_hex_corner_3d;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

pub struct Hexagon3D;

impl Hexagon3D {
    pub fn create_base(size: f32, starting_angle: f32, height_differences: [u8; 6]) -> Mesh {
        let top_vertices: [[f32; 3]; 6] = [
            get_hex_corner_3d(5, starting_angle, size, &height_differences),
            get_hex_corner_3d(4, starting_angle, size, &height_differences),
            get_hex_corner_3d(3, starting_angle, size, &height_differences),
            get_hex_corner_3d(2, starting_angle, size, &height_differences),
            get_hex_corner_3d(1, starting_angle, size, &height_differences),
            get_hex_corner_3d(0, starting_angle, size, &height_differences),
        ];

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::from(top_vertices))
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![
                [0.000093, 0.250046],
                [0.865957, 0.250046],
                [0.433025, 0.999907],
                [0.000093, 0.749953],
                [0.433025, 0.000093],
                [0.865957, 0.749954],
            ],
        )
        .with_inserted_indices(Indices::U16(vec![2, 4, 0, 0, 1, 2, 2, 3, 4, 4, 5, 0]))
    }
}
