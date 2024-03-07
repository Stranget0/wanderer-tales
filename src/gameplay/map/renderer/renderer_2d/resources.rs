use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::map::{renderer::components::MaterialKey, utils::hex_layout::HexLayout};

// #region Mesh
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum MeshKey {
    Hex,
    Player,
}

#[derive(Resource)]
pub struct MeshesStore(pub HashMap<MeshKey, Handle<Mesh>>);

impl Default for MeshesStore {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub fn init_meshes_store(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_map: ResMut<MeshesStore>,
    layout_query: Query<&HexLayout>,
) {
    let layout = layout_query.single();

    let entries: [(MeshKey, Mesh); 2] = [
        (MeshKey::Hex, RegularPolygon::new(layout.size.x, 6).into()),
        (MeshKey::Player, Circle::new(3.0).into()),
    ];

    for (key, mesh) in entries {
        mesh_map.0.insert(key, meshes.add(mesh));
    }
}
// #endregion

// #region Material

#[derive(Resource)]
pub struct MaterialStore(pub HashMap<MaterialKey, Handle<ColorMaterial>>);

impl Default for MaterialStore {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub fn init_materials_store(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut material_map: ResMut<MaterialStore>,
) {
    let colors = [
        (MaterialKey::Beach, Color::hex("#e1d76a")),
        (MaterialKey::Grass, Color::hex("#36b90b")),
        (MaterialKey::Forest, Color::hex("#054303")),
        (MaterialKey::Mountain, Color::hex("#302c2a")),
        (MaterialKey::Water, Color::hex("#0E499A")),
        (MaterialKey::Player, Color::hex("#f7f1d8")),
    ];

    for (key, color) in colors {
        let material_handle = materials.add(color.unwrap());
        material_map.0.insert(key, material_handle);
    }
}
// #endregion
