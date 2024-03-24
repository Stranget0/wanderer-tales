use crate::gameplay::map::utils::hex_layout::{get_hex_corner_2d, get_hex_corner_3d};
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
        (0..6)
            .map(|i: usize| get_hex_corner_3d(i, starting_angle, size, height_differences))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::map::utils::{
        hex_layout::get_hex_corner_2d, hex_map_item::Height, hex_vector::HEX_DIRECTIONS,
        lexigraphical_cycle::LexigraphicalCycle,
    };

    // #[test]
    // fn test_height_diff() {
    //     let height = Height(32);

    //     let hexes = [
    //         (HEX_DIRECTIONS[0], Height(32)),
    //         (HEX_DIRECTIONS[1], Height(32)),
    //         (HEX_DIRECTIONS[2], Height(32)),
    //         (HEX_DIRECTIONS[3], Height(32)),
    //         (HEX_DIRECTIONS[4], Height(32)),
    //         (HEX_DIRECTIONS[5], Height(33)),
    //     ];

    //     let height_diffs = hexes
    //         .iter()
    //         .map(|h| (h.1 .0 as i16 - height.0 as i16) as i8)
    //         .collect_vec();

    //     assert_eq!(height.0 + height_diffs[5], hexes[5].1 .0)
    // }

    // #[test]
    // fn test_height_corner() {
    //     let height = Height(32);

    //     let hexes = [
    //         (HEX_DIRECTIONS[0], Height(32)),
    //         (HEX_DIRECTIONS[1], Height(32)),
    //         (HEX_DIRECTIONS[2], Height(32)),
    //         (HEX_DIRECTIONS[3], Height(32)),
    //         (HEX_DIRECTIONS[4], Height(32)),
    //         (HEX_DIRECTIONS[5], Height(33)),
    //     ];

    //     let height_diffs = hexes
    //         .iter()
    //         .map(|(_, h)| (h.0 as i16 - height.0 as i16) as i8)
    //         .collect_vec();

    //     let height_diffs_cycle = LexigraphicalCycle::shiloah_minimal_rotation(height_diffs);

    //     // for (i, h) in height_diffs.iter().enumerate() {}
    // }
}
