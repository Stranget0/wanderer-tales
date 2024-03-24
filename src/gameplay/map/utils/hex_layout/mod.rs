use bevy::prelude::*;

use crate::utils::{positive_modulo, to_3d_space};

use super::{hex_vector::FractionalHexVector, layout_orientation::HexLayoutOrientation};

#[derive(Component, Debug)]
pub struct HexLayout {
    pub orientation: HexLayoutOrientation,
    pub size: Vec2,
    pub origin: Vec2,
}

impl HexLayout {
    pub fn hex_to_pixel(&self, h: &FractionalHexVector) -> Vec2 {
        let matrix = &self.orientation.matrix;
        let result = matrix.mul_vec2(Vec2::new(h.0, h.1)) * self.size;
        result + self.origin
    }

    pub fn pixel_to_hex(&self, p: Vec2) -> FractionalHexVector {
        let matrix = &self.orientation.matrix.inverse(); // Get the inverse matrix
        let pt = (p - self.origin) / self.size;
        let result = matrix.mul_vec2(pt);
        FractionalHexVector(result.x, result.y, -result.x - result.y)
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
    use bevy::math::vec2;

    use crate::gameplay::map::utils::{
        hex_vector::{FractionalHexVector, HexVector, HEX_DIRECTIONS},
        layout_orientation::POINTY_TOP_ORIENTATION,
    };

    use super::HexLayout;

    #[test]
    fn hex_to_pixel_test() {
        let input_output = vec![
            (HexVector::new(0, 0, 0), vec2(0.0, 0.0)),
            (HexVector::new(-1, 0, 1), vec2(-55.425625, 0.0)),
            (HexVector::new(0, -1, 1), vec2(-27.712812, -48.0)),
            (HexVector::new(1, -1, 0), vec2(27.712812, -48.0)),
            (HexVector::new(1, 0, -1), vec2(55.425625, 0.0)),
            (HexVector::new(0, 1, -1), vec2(27.712812, 48.0)),
            (HexVector::new(-1, 1, 0), vec2(-27.712812, 48.0)),
            (HexVector::new(-2, 1, 1), vec2(-83.138435, 48.0)),
            (HexVector::new(-2, 0, 2), vec2(-110.85125, 0.0)),
            (HexVector::new(-1, -1, 2), vec2(-83.138435, -48.0)),
            (HexVector::new(0, -2, 2), vec2(-55.425625, -96.0)),
            (HexVector::new(1, -2, 1), vec2(0.0, -96.0)),
            (HexVector::new(2, -2, 0), vec2(55.425625, -96.0)),
            (HexVector::new(2, -1, -1), vec2(83.138435, -48.0)),
            (HexVector::new(2, 0, -2), vec2(110.85125, 0.0)),
            (HexVector::new(1, 1, -2), vec2(83.138435, 48.0)),
            (HexVector::new(0, 2, -2), vec2(55.425625, 96.0)),
            (HexVector::new(-1, 2, -1), vec2(0.0, 96.0)),
            (HexVector::new(-2, 2, 0), vec2(-55.425625, 96.0)),
            (HexVector::new(-3, 2, 1), vec2(-110.85124, 96.0)),
            (HexVector::new(-3, 1, 2), vec2(-138.56406, 48.0)),
            (HexVector::new(-3, 0, 3), vec2(-166.27687, 0.0)),
            (HexVector::new(-2, -1, 3), vec2(-138.56406, -48.0)),
            (HexVector::new(-1, -2, 3), vec2(-110.85125, -96.0)),
            (HexVector::new(0, -3, 3), vec2(-83.138435, -144.0)),
            (HexVector::new(1, -3, 2), vec2(-27.71281, -144.0)),
            (HexVector::new(2, -3, 1), vec2(27.712814, -144.0)),
            (HexVector::new(3, -3, 0), vec2(83.138435, -144.0)),
            (HexVector::new(3, -2, -1), vec2(110.85124, -96.0)),
            (HexVector::new(3, -1, -2), vec2(138.56406, -48.0)),
            (HexVector::new(3, 0, -3), vec2(166.27687, 0.0)),
            (HexVector::new(2, 1, -3), vec2(138.56406, 48.0)),
            (HexVector::new(1, 2, -3), vec2(110.85125, 96.0)),
            (HexVector::new(0, 3, -3), vec2(83.138435, 144.0)),
            (HexVector::new(-1, 3, -2), vec2(27.71281, 144.0)),
            (HexVector::new(-2, 3, -1), vec2(-27.712814, 144.0)),
            (HexVector::new(-3, 3, 0), vec2(-83.138435, 144.0)),
        ];
        let layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(32.0, 32.0),
            origin: vec2(0.0, 0.0),
        };

        for (hex, pos) in input_output {
            let result = layout.hex_to_pixel(&hex.into());
            assert_eq!(result, pos);
        }
    }

    #[test]
    fn hex_to_pixel_test2() {
        let input_output = vec![
            (&HEX_DIRECTIONS[0], vec2(-173.20508, 300.0)),
            (&HEX_DIRECTIONS[1], vec2(173.20508, 300.0)),
            (&HEX_DIRECTIONS[2], vec2(346.41016, 0.0)),
            (&HEX_DIRECTIONS[3], vec2(173.20508, -300.0)),
            (&HEX_DIRECTIONS[4], vec2(-173.20508, -300.0)),
            (&HEX_DIRECTIONS[5], vec2(-346.41016, 0.0)),
        ];
        let layout = HexLayout {
            orientation: POINTY_TOP_ORIENTATION,
            size: vec2(200.0, 200.0),
            origin: vec2(0.0, 0.0),
        };

        for (hex, pos) in input_output {
            let fractional_hex = FractionalHexVector::from(hex);
            let result = layout.hex_to_pixel(&fractional_hex);
            println!("{}, {}", result.x, result.y);
            assert_eq!(result, pos);
        }
    }
}
