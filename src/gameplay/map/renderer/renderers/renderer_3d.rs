use bevy::{prelude::*, utils::hashbrown::HashMap};
use itertools::Itertools;

use crate::{
    gameplay::{
        map::{
            renderer::{
                components::{MaterialType, MeshType},
                debug::uv_debug_texture,
            },
            utils::hex_layout::HexLayout,
        },
        player::components::Rotation,
    },
    utils::EULER_ROT,
};

use super::{
    common::PRECOMPUTED_HEIGHT_CYCLES,
    meshes::Hexagon3D,
    traits::{CreateRenderBundles, RenderMap, RenderMapApi},
};

#[derive(Component)]
pub struct Renderer3D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialType, Handle<StandardMaterial>>,
    pub meshes_map: HashMap<MeshType, Handle<Mesh>>,
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
}

impl CreateRenderBundles<PbrBundle> for Renderer3D {
    fn create_render_bundle(
        &self,
        pos: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> (PbrBundle, Option<Vec<PbrBundle>>) {
        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        transform.rotation = Quat::from_euler(EULER_ROT, rotation.0.x, rotation.0.y, rotation.0.z);

        // if let MeshType::HexMapTile(height_cycle) = mesh_type {
        //     transform.rotate_y((height_cycle.rotation as f32 * 60.0).to_radians());
        // };

        let zero_type = MeshType::HexMapTile(default());

        let mesh_type_debug = match mesh_type {
            MeshType::HexMapTile(_) => &zero_type,
            _ => mesh_type,
        };

        let material = self
            .materials_map
            .get(material_type)
            .unwrap_or_else(|| {
                error!("Could not get material {:?}", material_type);
                self.materials_map
                    .get(&MaterialType::Debug)
                    .expect("Could not get debug material")
            })
            .clone();

        let mesh = self
            .meshes_map
            .get(mesh_type)
            .unwrap_or_else(|| {
                error!(
                    "Could not get mesh {:?} \n\tavailable: {:?}",
                    mesh_type,
                    self.meshes_map.keys().collect_vec()
                );
                self.meshes_map
                    .get(&MeshType::Debug)
                    .expect("Could not get debug mesh")
            })
            .clone();

        (
            PbrBundle {
                mesh,
                material,
                transform,
                ..Default::default()
            },
            None,
        )
    }
}

impl Renderer3D {
    pub fn new(
        layout: &HexLayout,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mut materials_map = HashMap::default();
        let mut meshes_map = HashMap::default();

        let debug_material = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        });

        let materials = [
            MaterialType::Beach,
            MaterialType::Grass,
            MaterialType::Forest,
            MaterialType::Mountain,
            MaterialType::Water,
            MaterialType::Player,
            MaterialType::Debug,
        ];

        for key in materials {
            materials_map.insert(key, debug_material.clone());
        }

        let entries: Vec<(MeshType, Mesh)> = vec![
            (MeshType::Player, Sphere::new(layout.size.x).into()),
            (MeshType::Debug, Sphere::new(0.1).into()),
        ];

        for (key, mesh) in entries {
            meshes_map.insert(key, meshes.add(mesh));
        }
        for height_cycle in PRECOMPUTED_HEIGHT_CYCLES {
            meshes_map.insert(
                MeshType::HexMapTile(height_cycle),
                meshes.add(Hexagon3D::create_base(
                    layout.size.x,
                    layout.orientation.starting_angle,
                    &height_cycle,
                )),
            );
        }

        Self {
            renders_map: RenderMap::default(),
            materials_map,
            meshes_map,
        }
    }
}
