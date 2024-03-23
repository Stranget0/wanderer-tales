use crate::gameplay::map::utils::{
    hex_layout::get_hex_corner_3d, hex_vector::iterators::HexCorners,
};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use itertools::Itertools;

pub struct Hexagon3D;

impl Hexagon3D {
    pub fn create_base(size: f32, starting_angle: f32, height_differences: &[i8; 6]) -> Mesh {
        let mut top_vertices: [[f32; 3]; 6] =
            Self::get_top_vertices(starting_angle, size, height_differences);
        top_vertices.reverse();

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

    pub fn get_top_vertices(
        starting_angle: f32,
        size: f32,
        height_differences: &[i8; 6],
    ) -> [[f32; 3]; 6] {
        HexCorners {
            corner: 0,
            size,
            starting_angle,
        }
        .take(6)
        .map(|(i, [x, y])| {
            [
                x,
                y,
                ((height_differences[(i + 9) % 6] + height_differences[(i + 10) % 6]) as f32 / 3.0),
            ]
        })
        .collect_vec()
        .try_into()
        .unwrap()
    }
}
