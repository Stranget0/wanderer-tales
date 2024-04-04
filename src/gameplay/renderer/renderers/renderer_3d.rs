use bevy::{prelude::*, utils::hashbrown::HashMap};

use super::{
    meshes::Hexagon3D,
    traits::{CreateRenderBundles, RenderMap, RenderMapApi},
};
use crate::gameplay::data_source_layer::{map::components::Rotation, utils::HexLayout};
use crate::gameplay::renderer::components::*;
use crate::gameplay::renderer::debug::uv_debug_texture;

#[derive(Component, Default)]
pub struct Renderer3D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialType, AssetId<StandardMaterial>>,
    pub meshes_map: HashMap<MeshType, AssetId<Mesh>>,
}

impl RenderMapApi for Renderer3D {
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

impl CreateRenderBundles<PbrBundle, StandardMaterial> for Renderer3D {
    fn create_render_bundle(
        &mut self,
        pos: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,

        layout: &HexLayout,
        asset_server: &Res<AssetServer>,
    ) -> PbrBundle {
        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        transform.rotation = rotation.0;

        let material = self.get_or_create_material(material_type, asset_server);
        let mesh = self.get_or_create_mesh(mesh_type, layout, asset_server);

        PbrBundle {
            mesh,
            material,
            transform,
            ..Default::default()
        }
    }
}

impl Renderer3D {
    fn get_or_create_material(
        &mut self,
        material_type: &MaterialType,
        asset_server: &Res<AssetServer>,
    ) -> Handle<StandardMaterial> {
        if let Some(handle) = self
            .materials_map
            .get(material_type)
            .and_then(|asset_id| asset_server.get_id_handle(*asset_id))
        {
            return handle.clone();
        }
        debug!("Create new material 3d");

        let material = match material_type {
            MaterialType::Grass => StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/grass.jpg")),
                ..default()
            },
            _ => StandardMaterial {
                base_color_texture: Some(asset_server.add(uv_debug_texture())),
                ..default()
            },
        };

        let handle = asset_server.add(material);
        self.materials_map.insert(*material_type, handle.id());

        handle
    }

    fn get_or_create_mesh(
        &mut self,
        mesh_type: &MeshType,
        layout: &HexLayout,
        asset_server: &Res<AssetServer>,
    ) -> Handle<Mesh> {
        if let Some(handle) = self
            .meshes_map
            .get(mesh_type)
            .and_then(|asset_id| asset_server.get_id_handle(*asset_id))
        {
            return handle.clone();
        }

        debug!("Create new mesh 3d");
        let mesh: Mesh = match mesh_type {
            MeshType::HexMapTile(height_cycle) => Hexagon3D::create_base(
                layout.size.x,
                layout.orientation.starting_angle,
                height_cycle,
            ),
            MeshType::Player => Sphere::new(0.3).into(),
            MeshType::Debug => Sphere::new(0.1).into(),
        };

        let handle = asset_server.add(mesh);
        self.meshes_map.insert(*mesh_type, handle.id());

        handle
    }

    // let debug_material = materials.add(StandardMaterial {
    //     base_color_texture: Some(images.add(uv_debug_texture())),
    //     ..default()
    // });

    // let materials = [
    //     MaterialType::Beach,
    //     MaterialType::Grass,
    //     MaterialType::Forest,
    //     MaterialType::Mountain,
    //     MaterialType::Water,
    //     MaterialType::Player,
    //     MaterialType::Debug,
    // ];

    // for key in materials {
    //     materials_map.insert(key, debug_material.clone());
    // }

    // let entries: Vec<(MeshType, Mesh)> = vec![
    //     (MeshType::Player, Sphere::new(0.3).into()),
    //     (MeshType::Debug, Sphere::new(0.1).into()),
    // ];

    // for (key, mesh) in entries {
    //     meshes_map.insert(key, meshes.add(mesh));
    // }
    // for height_cycle in PRECOMPUTED_HEIGHT_CYCLES {
    //     meshes_map.insert(
    //         MeshType::HexMapTile(height_cycle),
    // meshes.add(Hexagon3D::create_base(
    //             layout.size.x,
    //             layout.orientation.starting_angle,
    //             &height_cycle,
    //         )),
    //     );
    // }
}
