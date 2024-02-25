use bevy::prelude::*;

use super::hex_vector::HexVector;

#[derive(Component, Clone, Debug)]
pub struct HexMapItem {
    pub pos: HexVector,
    pub biome: Biome,
}

#[derive(Component, Clone, Debug, Copy)]
pub enum Biome {
    Grass,
    Forest,
    Mountain,
}
