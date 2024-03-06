use bevy::{prelude::*, utils::HashMap};
use noise::permutationtable::PermutationTable;

use crate::gameplay::map::utils::hex_vector::HexVector;

#[derive(Resource)]
pub struct MapData {
    pub hex_to_entity: HashMap<HexVector, Entity>,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            hex_to_entity: HashMap::new(),
        }
    }
}

#[derive(Resource)]
pub struct SeedTable {
    pub table: PermutationTable,
}

impl Default for SeedTable {
    fn default() -> Self {
        Self {
            table: PermutationTable::new(0),
        }
    }
}
