use bevy::prelude::*;

use super::hex_vector::HexVector;

#[derive(Component, Clone, Debug)]
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
pub struct Height(pub u8);
