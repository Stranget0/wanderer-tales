use crate::utils::*;
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
        let top_vertices: [[f32; 3]; 6] =
            Self::get_top_vertices(starting_angle, size, height_differences);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::from(top_vertices))
        // .with_inserted_attribute(
        //     Mesh::ATTRIBUTE_NORMAL,
        //     vec![
        //         [0.0, 1.0, 0.0],
        //         [0.0, 1.0, 0.0],
        //         [0.0, 1.0, 0.0],
        //         [0.0, 1.0, 0.0],
        //         [0.0, 1.0, 0.0],
        //         [0.0, 1.0, 0.0],
        //     ],
        // )
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
        .with_inserted_indices(Indices::U16(vec![2, 4, 0, 0, 1, 2, 2, 3, 4, 4, 5, 0]));

        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        mesh
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

pub fn get_hex_corner_2d(index: usize, starting_angle: f32, size: f32) -> [f32; 2] {
    let angle: f32 = 2.0 * std::f32::consts::PI * (starting_angle - index as f32 - 2.0) / 6.0;

    [size * angle.sin(), size * angle.cos()]
}

pub fn get_hex_corner_3d(
    index: usize,
    starting_angle: f32,
    size: f32,
    height_differences: &[i8; 6],
) -> [f32; 3] {
    let [x, y] = get_hex_corner_2d(index, starting_angle, size);

    let z = get_hex_corner_z(get_corner_heights(height_differences, index));

    to_3d_space(x, y, z)
}

pub fn get_corner_heights(height_differences: &[i8; 6], index: usize) -> [&i8; 2] {
    [
        &height_differences[positive_modulo(index, 6) as usize],
        &height_differences[positive_modulo(index as i16 - 1, 6) as usize],
    ]
}

pub fn get_hex_corner_z(heights: [&i8; 2]) -> f32 {
    // base is always 0
    let mut sum = 0;

    for h in heights {
        sum += h;
    }

    f32::from(sum) / 3.0
}

#[cfg(test)]
mod tests {
    use crate::gameplay::map::utils::*;

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
