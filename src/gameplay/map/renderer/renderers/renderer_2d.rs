use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::HashMap,
};

use crate::gameplay::map::{
    renderer::components::{MaterialType, MeshType},
    utils::{hex_layout::HexLayout, lexigraphical_cycle::LexigraphicalCycle},
};

use super::traits::{CreateRenderBundle, RenderMap, RenderMapApi};

#[derive(Component)]
pub struct Renderer2D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialType, Handle<ColorMaterial>>,
    pub meshes_map: HashMap<MeshType, Mesh2dHandle>,
}

impl RenderMapApi for Renderer2D {
    fn get_render_item(&self, source_entity: &Entity) -> Option<&Entity> {
        self.renders_map.get_render_item(source_entity)
    }

    fn remove_render_item(&mut self, source_entity: &Entity) -> Option<Entity> {
        self.renders_map.remove_render_item(source_entity)
    }

    fn link_source_item(
        &mut self,
        source_entity: &Entity,
        render_entity: &Entity,
    ) -> Option<Entity> {
        self.renders_map
            .link_source_item(source_entity, render_entity)
    }
}

impl CreateRenderBundle<MaterialMesh2dBundle<ColorMaterial>> for Renderer2D {
    fn create_render_bundle(
        &self,
        pos: &Vec3,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        let transform = Transform::from_xyz(
            pos.x,
            pos.y,
            match mesh_type {
                MeshType::HexMapTile(_) => 0.0,
                MeshType::Player => 1.0,
                MeshType::Debug => 2.0,
            },
        );

        let material = self
            .materials_map
            .get(material_type)
            .unwrap_or_else(|| self.materials_map.get(&MaterialType::Debug).unwrap())
            .clone();

        let mesh = self
            .meshes_map
            .get(&match mesh_type {
                MeshType::HexMapTile(_) => {
                    MeshType::HexMapTile(LexigraphicalCycle::shiloah_minimal_rotation(&[
                        0, 0, 0, 0, 0, 0,
                    ]))
                }
                _ => mesh_type.clone(),
            })
            .unwrap_or_else(|| self.meshes_map.get(&MeshType::Debug).unwrap())
            .clone();

        MaterialMesh2dBundle {
            mesh,
            material,
            transform,
            ..Default::default()
        }
    }
}

impl Renderer2D {
    pub fn new(
        layout: &HexLayout,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mut materials_map = HashMap::default();
        let mut meshes_map = HashMap::default();

        let colors = [
            (MaterialType::Beach, Color::hex("#e1d76a")),
            (MaterialType::Grass, Color::hex("#36b90b")),
            (MaterialType::Forest, Color::hex("#054303")),
            (MaterialType::Mountain, Color::hex("#302c2a")),
            (MaterialType::Water, Color::hex("#0E499A")),
            (MaterialType::Player, Color::hex("#f7f1d8")),
            (MaterialType::Debug, Color::hex("#ea00ff")),
        ];

        for (key, color) in colors {
            let material_handle = materials.add(color.unwrap());
            materials_map.insert(key, material_handle);
        }

        let entries: [(MeshType, Mesh); 3] = [
            (
                MeshType::HexMapTile(LexigraphicalCycle::shiloah_minimal_rotation(&[
                    0, 0, 0, 0, 0, 0,
                ])),
                RegularPolygon::new(layout.size.x, 6).into(),
            ),
            (MeshType::Player, Circle::new(3.0).into()),
            (
                MeshType::Debug,
                RegularPolygon::new(layout.size.x, 3).into(),
            ),
        ];

        for (key, mesh) in entries {
            meshes_map.insert(key, Mesh2dHandle(meshes.add(mesh)));
        }

        Self {
            renders_map: RenderMap::default(),
            materials_map,
            meshes_map,
        }
    }
}
