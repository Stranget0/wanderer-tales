use bevy::{math::vec2, prelude::*};

const SQRT_3: f32 = 1.7320508;

pub struct HexLayoutOrientation {
    pub f0: f32,
    pub f1: f32,
    pub f2: f32,
    pub f3: f32,
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub b3: f32,
    pub starting_angle: i16,
}

pub const POINTY_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    f0: SQRT_3,
    f1: SQRT_3 / 2.0,
    f2: 0.0,
    f3: 3.0 / 2.0,
    b0: SQRT_3 / 3.0,
    b1: -1.0 / 3.0,
    b2: 0.0,
    b3: 2.0 / 3.0,
    starting_angle: 0,
};

pub const FLAT_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    f0: 3.0 / 2.0,
    f1: 0.0,
    f2: SQRT_3 / 2.0,
    f3: SQRT_3,
    b0: 2.0 / 3.0,
    b1: 0.0,
    b2: -1.0 / 3.0,
    b3: SQRT_3 / 3.0,
    starting_angle: -50,
};
