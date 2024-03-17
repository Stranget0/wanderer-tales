use crate::gameplay::map::utils::hex_vector::iterators::HexCorners;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

pub struct Hexagon3D;

impl Hexagon3D {
    pub fn create_base(size: f32, starting_angle: f32) -> Mesh {
        let mut corners_clockwise = HexCorners {
            corner: 0,
            size,
            starting_angle,
        }
        .take(6);

        let top_vertices: [[f32; 3]; 6] = [
            corners_clockwise.next().unwrap(),
            corners_clockwise.next().unwrap(),
            corners_clockwise.next().unwrap(),
            corners_clockwise.next().unwrap(),
            corners_clockwise.next().unwrap(),
            corners_clockwise.next().unwrap(),
        ]
        .map(|c| [c[0], c[1], 0.0]);

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                top_vertices[0],
                top_vertices[1],
                top_vertices[2],
                top_vertices[3],
                top_vertices[4],
                top_vertices[5],
            ],
        )
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
