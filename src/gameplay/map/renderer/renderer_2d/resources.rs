use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::map::{
    renderer::{components::RenderGroup, utils::MaterialKey},
    utils::hex_layout::HexLayout,
};

// #region Mesh
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum MeshKey2d {
    Hex,
    Character,
}

#[derive(Resource)]
pub(crate) struct MeshesStore2d(pub HashMap<MeshKey2d, Handle<Mesh>>);

impl Default for MeshesStore2d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub(crate) fn init_meshes_store(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_map: ResMut<MeshesStore2d>,
    layout_query: Query<(&HexLayout, &RenderGroup)>,
) {
    for (layout, render_pass) in layout_query.iter() {
        if render_pass != &RenderGroup::PreviewMap2D {
            continue;
        }

        let entries: [(MeshKey2d, Mesh); 2] = [
            (MeshKey2d::Hex, RegularPolygon::new(layout.size.x, 6).into()),
            (MeshKey2d::Character, Circle::new(3.0).into()),
        ];

        for (key, mesh) in entries {
            mesh_map.0.insert(key, meshes.add(mesh));
        }
    }
}
// #endregion

// #region Material

#[derive(Resource)]
pub(crate) struct MaterialStore2d(pub HashMap<MaterialKey, Handle<ColorMaterial>>);

impl Default for MaterialStore2d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub(crate) fn init_materials_store(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut material_map: ResMut<MaterialStore2d>,
) {
    let colors = [
        (MaterialKey::Beach, Color::hex("#e1d76a")),
        (MaterialKey::Grass, Color::hex("#36b90b")),
        (MaterialKey::Forest, Color::hex("#054303")),
        (MaterialKey::Mountain, Color::hex("#302c2a")),
        (MaterialKey::Water, Color::hex("#0E499A")),
        (MaterialKey::Player, Color::hex("#f7f1d8")),
        (MaterialKey::Debug, Color::hex("#ea00ff")),
    ];

    for (key, color) in colors {
        let material_handle = materials.add(color.unwrap());
        material_map.0.insert(key, material_handle);
    }
}
// #endregion

// #region source to render
#[derive(Resource, Debug)]
pub(crate) struct SourceToRenderStore2d(pub HashMap<u32, Entity>);

impl Default for SourceToRenderStore2d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
// #endregion
