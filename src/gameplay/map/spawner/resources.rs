use bevy::{prelude::*, utils::hashbrown::HashMap};
use noise::permutationtable::PermutationTable;

use crate::gameplay::map::utils::hex_vector::HexVector;

#[derive(Resource)]
pub struct HexToMapSourceEntity(pub HashMap<HexVector, Entity>);

impl Default for HexToMapSourceEntity {
    fn default() -> Self {
        Self(HashMap::new())
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
