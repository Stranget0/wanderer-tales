use bevy::{asset::UntypedAssetId, pbr::ExtendedMaterial, prelude::*, utils::hashbrown::HashMap};

use super::{
    meshes::Hexagon3D,
    traits::{RenderMap, RenderMapApi, SpawnRenderBundle},
};
use crate::gameplay::renderer::components::*;
use crate::gameplay::renderer::debug::uv_debug_texture;
use crate::{
    gameplay::data_source_layer::{map::components::Rotation, utils::HexLayout},
    utils::WorldAlignedExtension,
};

#[derive(Component, Default)]
pub struct Renderer3D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialType, UntypedAssetId>,
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

impl SpawnRenderBundle for Renderer3D {
    fn spawn_render_item(
        &mut self,
        commands: &mut Commands,
        source_entity: &Entity,
        pos: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,

        (layout_entity, layout): (&Entity, &HexLayout),
        asset_server: &Res<AssetServer>,
    ) {
        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        transform.rotation = rotation.0;

        let mesh = self.get_or_create_mesh(mesh_type, layout, asset_server);

        let render_entity = match material_type {
            MaterialType::Grass => {
                let material = self
                    .get_or_create_material::<ExtendedMaterial<StandardMaterial, WorldAlignedExtension>>(
                        material_type,
                        asset_server,
                    );

                commands
                    .spawn(MaterialMeshBundle {
                        mesh,
                        material,
                        transform,
                        ..Default::default()
                    })
                    .id()
            }

            _ => {
                let material =
                    self.get_or_create_material::<StandardMaterial>(material_type, asset_server);

                commands
                    .spawn(MaterialMeshBundle {
                        mesh,
                        material,
                        transform,
                        ..Default::default()
                    })
                    .id()
            }
        };

        self.link_source_item(source_entity, &render_entity);

        commands.entity(*layout_entity).add_child(render_entity);
    }
}

impl Renderer3D {
    fn get_or_create_material<M: Material>(
        &mut self,
        material_type: &MaterialType,
        asset_server: &Res<AssetServer>,
    ) -> Handle<M> {
        if let Some(handle) = self
            .materials_map
            .get(material_type)
            .and_then(|asset_id| asset_server.get_id_handle_untyped(*asset_id).clone())
        {
            return handle.typed::<M>();
        }

        debug!("Create new material 3d");

        let handle = match material_type {
            MaterialType::Grass => asset_server
                .add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(asset_server.load("textures/grass.jpg")),
                        ..default()
                    },
                    extension: WorldAlignedExtension::new(0.1),
                })
                .untyped(),
            _ => asset_server
                .add(StandardMaterial {
                    base_color_texture: Some(asset_server.add(uv_debug_texture())),
                    ..default()
                })
                .untyped(),
        };

        self.materials_map.insert(*material_type, handle.id());

        handle.typed::<M>()
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
}
