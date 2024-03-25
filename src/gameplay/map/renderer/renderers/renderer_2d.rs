use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::HashMap,
};

use crate::{
    gameplay::{
        components::Rotation,
        map::{
            renderer::components::{MaterialType, MeshType},
            utils::hex_layout::HexLayout,
        },
    },
    utils::UP,
};

use super::traits::{CreateRenderBundles, RenderMap, RenderMapApi};

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

impl CreateRenderBundles<MaterialMesh2dBundle<ColorMaterial>> for Renderer2D {
    fn create_render_bundle(
        &self,
        pos_3d: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> (
        MaterialMesh2dBundle<ColorMaterial>,
        Option<Vec<MaterialMesh2dBundle<ColorMaterial>>>,
    ) {
        let pos = zero_up_vec(pos_3d) + type_to_up(mesh_type);

        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        transform.rotation = rotation.0;

        let material = self
            .materials_map
            .get(material_type)
            .unwrap_or_else(|| self.materials_map.get(&MaterialType::Debug).unwrap())
            .clone();

        let mesh = self
            .meshes_map
            .get(&match mesh_type {
                MeshType::HexMapTile(_) => MeshType::HexMapTile(default()),
                _ => mesh_type.clone(),
            })
            .unwrap_or_else(|| self.meshes_map.get(&MeshType::Debug).unwrap())
            .clone();

        (
            MaterialMesh2dBundle {
                mesh,
                material,
                transform,
                ..Default::default()
            },
            None,
        )
    }
}

fn type_to_up(mesh_type: &MeshType) -> Vec3 {
    UP * match mesh_type {
        MeshType::HexMapTile(_) => 0.0,
        MeshType::Player => 1.0,
        MeshType::Debug => 2.0,
    }
}

fn zero_up_vec(pos_ref: &Vec3) -> Vec3 {
    let base_pos = *pos_ref;

    (Vec3::ONE - UP) * base_pos
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
                MeshType::HexMapTile(default()),
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
