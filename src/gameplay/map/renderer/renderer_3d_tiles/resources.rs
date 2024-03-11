use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::map::{
    renderer::{components::RenderGroup, debug::uv_debug_texture, utils::MaterialKey},
    utils::hex_layout::HexLayout,
};

use super::mesh::Hexagon3D;

// #region Mesh
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum MeshKey3d {
    Hex,
    Character,
}

#[derive(Resource)]
pub(crate) struct MeshesStore3d(pub HashMap<MeshKey3d, Handle<Mesh>>);

impl Default for MeshesStore3d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub(crate) fn init_meshes_store(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_map: ResMut<MeshesStore3d>,
    layout_query: Query<(&HexLayout, &RenderGroup)>,
) {
    for (layout, render_group) in layout_query.iter() {
        if render_group != &RenderGroup::Gameplay3D {
            continue;
        }
        let entries: [(MeshKey3d, Mesh); 2] = [
            (
                MeshKey3d::Hex,
                Hexagon3D::create_mesh(layout.size.x, layout.orientation.starting_angle),
            ),
            (MeshKey3d::Character, Sphere::new(layout.size.x).into()),
        ];

        for (key, mesh) in entries {
            mesh_map.0.insert(key, meshes.add(mesh));
        }
    }
}
// #endregion

// #region Material

#[derive(Resource)]
pub(crate) struct MaterialStore3d(pub HashMap<MaterialKey, Handle<StandardMaterial>>);

impl Default for MaterialStore3d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub(crate) fn init_materials_store(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut material_map: ResMut<MaterialStore3d>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let materials = [
        (MaterialKey::Beach, Color::hex("#e1d76a")),
        (MaterialKey::Grass, Color::hex("#36b90b")),
        (MaterialKey::Forest, Color::hex("#054303")),
        (MaterialKey::Mountain, Color::hex("#302c2a")),
        (MaterialKey::Water, Color::hex("#0E499A")),
        (MaterialKey::Player, Color::hex("#f7f1d8")),
    ];

    for (key, _) in materials {
        material_map.0.insert(key, debug_material.clone());
    }
}
// #endregion

// #region source to render
#[derive(Resource, Debug)]
pub(crate) struct SourceToRenderStore3d(pub HashMap<u32, Entity>);

impl Default for SourceToRenderStore3d {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
// #endregion
