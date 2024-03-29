use bevy::prelude::*;

use crate::gameplay::data_source_layer::map::components::*;
use crate::gameplay::renderer::components::*;

#[derive(Bundle, Clone, Debug)]
pub struct HexMapTileBundle {
    pub pos: HexPosition,
    pub biome: Biome,
    pub tile_height: TileHeight,
    pub height: Height,
    pub material_type: MaterialType,
    pub mesh_type: MeshType,
}

#[derive(Component, Clone, Debug, Copy)]
pub enum Biome {
    Grass,
    Forest,
}

#[derive(Component, Clone, Debug)]
pub struct TileHeight {
    // [0, 255]
    pub midpoint: u8,
    // [-1, 1]
    pub offset: f32,
}

impl TileHeight {
    pub fn get_height(&self) -> u8 {
        (self.midpoint as i16 / 4 + (self.offset * 2.0) as i16 * 10) as u8
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

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Height(pub u8);
