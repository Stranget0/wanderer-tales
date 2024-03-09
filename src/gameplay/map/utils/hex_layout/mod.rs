use bevy::prelude::*;

use super::{
    hex_vector::{FractionalHexVector, HexVector},
    layout_orientation::HexLayoutOrientation,
};

#[derive(Resource, Debug)]
pub struct HexLayout {
    pub orientation: HexLayoutOrientation,
    pub size: Vec2,
    pub origin: Vec2,
}
impl HexLayout {
    pub fn hex_to_pixel(&self, h: &HexVector) -> Vec2 {
        let matrix = &self.orientation;

        let x = (matrix.f0 * h.0 as f32 + matrix.f1 * h.1 as f32) * self.size.x;
        let y = (matrix.f2 * h.0 as f32 + matrix.f3 * h.1 as f32) * self.size.y;
        Vec2::from_array([x, y])
    }

    pub fn pixel_to_hex(&self, p: Vec2) -> FractionalHexVector {
        let matrix = &self.orientation;
        let pt = Vec2::new(
            (p.x - self.origin.x) / self.size.x,
            (p.y - self.origin.y) / self.size.y,
        );

        let q: f32 = matrix.b0 * pt.x + matrix.b1 * pt.y;
        let r: f32 = matrix.b2 * pt.x + matrix.b3 * pt.y;

        FractionalHexVector(q, r, -q - r)
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::vec2;

    use crate::gameplay::map::utils::{
        hex_vector::HexVector, layout_orientation::POINTY_TOP_ORIENTATION,
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
            let result = layout.hex_to_pixel(&hex);
            assert_eq!(result, pos);
        }
    }
}
