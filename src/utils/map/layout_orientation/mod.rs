use bevy::{math::vec2, prelude::*};

const SQRT_3: f32 = 1.7320508075688772;
const HEX_TO_PIXEL_POINTY: Mat2 = Mat2::from_cols(vec2(SQRT_3, 0.0), vec2(SQRT_3 / 2.0, 3.0 / 2.0));
const HEX_TO_PIXEL_FLAT: Mat2 = Mat2::from_cols(vec2(3.0 / 2.0, SQRT_3 / 2.0), vec2(0.0, SQRT_3));

pub const POINTY_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    hex_to_pixel: HEX_TO_PIXEL_POINTY,
    pixel_to_hex: None,
};

pub const FLAT_TOP_ORIENTATION: HexLayoutOrientation = HexLayoutOrientation {
    hex_to_pixel: HEX_TO_PIXEL_FLAT,
    pixel_to_hex: None,
};

pub struct HexLayoutOrientation {
    pub hex_to_pixel: Mat2,
    pub pixel_to_hex: Option<Mat2>,
}

impl HexLayoutOrientation {
    pub fn pixel_to_hex(&mut self) -> Mat2 {
        match self.pixel_to_hex {
            Some(inv) => inv,
            None => {
                let inverse = self.hex_to_pixel.inverse();
                self.pixel_to_hex = Some(inverse);

                inverse
            }
        }
    }
}
