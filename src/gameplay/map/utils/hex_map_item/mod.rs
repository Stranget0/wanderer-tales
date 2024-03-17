use bevy::prelude::*;

use crate::gameplay::{
    map::renderer::components::{MaterialType, MeshType},
    player::components::HexPosition,
};

#[derive(Bundle, Clone, Debug)]
pub struct HexMapTileBundle {
    pub pos: HexPosition,
    pub biome: Biome,
    pub height: Height,
    pub mesh_type: MeshType,
    pub material_type: MaterialType,
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
        ((self.midpoint as f32) - self.offset) as i16
    }

    pub fn get_height(&self) -> u8 {
        (self.midpoint as i16 + (self.offset * 35.0) as i16) as u8
    }

    pub fn get_material(&self) -> MaterialType {
        if self.offset < -0.5 {
            return MaterialType::Water;
        }

        if self.offset < -0.2 {
            return MaterialType::Beach;
        }

        if self.offset < 0.3 {
            return MaterialType::Grass;
        }

        if self.offset < 0.65 {
            return MaterialType::Forest;
        }

        MaterialType::Mountain
    }
}
