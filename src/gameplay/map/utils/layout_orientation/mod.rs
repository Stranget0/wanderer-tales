use bevy::math::{Mat2, Vec2};

const SQRT_3: f32 = 1.7320508;

// #[derive(Debug)]
// pub struct HexLayoutOrientation {
//     pub f0: f32,
//     pub f1: f32,
//     pub f2: f32,
//     pub f3: f32,
//     pub b0: f32,
//     pub b1: f32,
//     pub b2: f32,
//     pub b3: f32,
//     pub starting_angle: f32,
// }

// pub const POINTY_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
//     f0: SQRT_3,
//     f1: SQRT_3 / 2.0,
//     f2: 0.0,
//     f3: 3.0 / 2.0,
//     b0: SQRT_3 / 3.0,
//     b1: -1.0 / 3.0,
//     b2: 0.0,
//     b3: 2.0 / 3.0,
//     starting_angle: 0.0,
// };

// pub const FLAT_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
//     f0: 3.0 / 2.0,
//     f1: 0.0,
//     f2: SQRT_3 / 2.0,
//     f3: SQRT_3,
//     b0: 2.0 / 3.0,
//     b1: 0.0,
//     b2: -1.0 / 3.0,
//     b3: SQRT_3 / 3.0,
//     starting_angle: 0.5,
// };
#[derive(Debug)]
pub struct HexLayoutOrientation {
    pub matrix: Mat2,
    pub starting_angle: f32,
}

pub const POINTY_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    matrix: Mat2::from_cols_array(&[SQRT_3, 0.0, SQRT_3 / 2.0, 3.0 / 2.0]),
    starting_angle: 0.0,
};

pub const FLAT_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    matrix: Mat2::from_cols(Vec2::new(3.0 / 2.0, 0.0), Vec2::new(SQRT_3 / 2.0, SQRT_3)),
    starting_angle: 0.5,
};
