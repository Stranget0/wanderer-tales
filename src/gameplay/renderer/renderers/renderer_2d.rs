use crate::gameplay::data_source_layer::utils::HexLayout;
use crate::{
    gameplay::{data_source_layer::map::components::Rotation, renderer::components::*},
    utils::UP,
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::HashMap,
};

use super::traits::{CreateRenderBundles, RenderMap, RenderMapApi};

#[derive(Component, Default)]
pub struct Renderer2D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialType, AssetId<ColorMaterial>>,
    pub meshes_map: HashMap<MeshType, AssetId<Mesh>>,
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

    fn count(&self) -> usize {
        self.renders_map.count()
    }
}

impl CreateRenderBundles<MaterialMesh2dBundle<ColorMaterial>, ColorMaterial> for Renderer2D {
    fn create_render_bundle(
        &mut self,
        pos_3d: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,
        layout: &HexLayout,
        asset_server: &Res<AssetServer>,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        let pos = zero_up_vec(pos_3d) + type_to_up(mesh_type);

        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        transform.rotation = rotation.0;

        let material = self.get_or_create_material(material_type, asset_server);

        let mesh = self.get_or_create_mesh(mesh_type, layout, asset_server);

        MaterialMesh2dBundle {
            mesh,
            material,
            transform,
            ..Default::default()
        }
    }
}

impl Renderer2D {
    fn get_or_create_material(
        &mut self,
        material_type: &MaterialType,
        asset_server: &Res<AssetServer>,
    ) -> Handle<ColorMaterial> {
        if let Some(handle) = self
            .materials_map
            .get(material_type)
            .and_then(|asset_id| asset_server.get_id_handle(*asset_id))
        {
            return handle.clone();
        }
        let material = match material_type {
            MaterialType::Beach => Color::hex("#e1d76a"),
            MaterialType::Grass => Color::hex("#36b90b"),
            MaterialType::Forest => Color::hex("#054303"),
            MaterialType::Mountain => Color::hex("#302c2a"),
            MaterialType::Water => Color::hex("#0E499A"),
            MaterialType::Player => Color::hex("#f7f1d8"),
            MaterialType::Debug => Color::hex("#ea00ff"),
        }
        .unwrap_or_else(|_| panic!("Invalid color definition for {:?}", material_type));

        let handle = asset_server.add(ColorMaterial::from(material));
        self.materials_map.insert(*material_type, handle.id());

        handle
    }

    fn get_or_create_mesh(
        &mut self,
        mesh_type: &MeshType,
        layout: &HexLayout,
        asset_server: &Res<AssetServer>,
    ) -> Mesh2dHandle {
        if let Some(handle) = self
            .meshes_map
            .get(mesh_type)
            .and_then(|asset_id| asset_server.get_id_handle(*asset_id))
        {
            return Mesh2dHandle(handle.clone());
        }

        debug!("Create new mesh 2d");
        let mesh: Mesh = match mesh_type {
            MeshType::HexMapTile(_) => RegularPolygon::new(layout.size.x, 6).into(),
            MeshType::Player => Circle::new(3.0).into(),
            MeshType::Debug => RegularPolygon::new(layout.size.x, 3).into(),
        };

        let handle = asset_server.add(mesh);
        self.meshes_map.insert(*mesh_type, handle.id());

        Mesh2dHandle(handle)
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
