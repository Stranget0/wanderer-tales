use bevy::prelude::*;

use crate::gameplay::map::renderer::renderer_2d::resources::MaterialKey;

use super::hex_vector::HexVector;

#[derive(Bundle, Clone, Debug)]
pub struct HexMapItemBundle {
    pub pos: HexVector,
    pub biome: Biome,
    pub height: Height,
}

#[derive(Component, Clone, Debug, Copy)]
pub enum Biome {
    Grass,
    Forest,
}

#[derive(Component, Clone, Debug, Copy)]
pub struct Height {
    // [0, 255]
    pub midpoint: u8,
    // [-1, 1]
    pub offset: f32,
}

impl Height {
    fn get_difference(&self) -> i16 {
        ((self.midpoint as f32) - (self.offset as f32)) as i16
    }

    pub fn get_height(&self) -> u8 {
        (self.midpoint as i16 + (self.offset * 35.0) as i16) as u8
    }

    pub fn get_material(&self) -> MaterialKey {
        if self.offset < -0.5 {
            return MaterialKey::Water;
        }

        if self.offset < -0.2 {
            return MaterialKey::Beach;
        }

        if self.offset < 0.3 {
            return MaterialKey::Grass;
        }

        if self.offset < 0.65 {
            return MaterialKey::Forest;
        }

        MaterialKey::Mountain
    }
}
