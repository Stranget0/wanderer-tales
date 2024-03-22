use bevy::{prelude::*, utils::hashbrown::HashMap};
use itertools::Itertools;

use crate::gameplay::map::{
    renderer::{
        components::{MaterialType, MeshType},
        debug::uv_debug_texture,
    },
    utils::{hex_layout::HexLayout, lexigraphical_cycle::LexigraphicalCycle},
};

use super::{
    common::PRECOMPUTED_HEIGHT_DIFF,
    meshes::Hexagon3D,
    traits::{CreateRenderBundle, RenderMap, RenderMapApi},
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

impl CreateRenderBundle<PbrBundle> for Renderer3D {
    fn create_render_bundle(
        &self,
        pos: &Vec3,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> PbrBundle {
        let mut transform = Transform::from_xyz(pos.x, pos.y, pos.z);

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
            .get(&match mesh_type {
                MeshType::HexMapTile(heigh_diffs) => {
                    let normalized_diffs = LexigraphicalCycle::shiloah_minimal_rotation(
                        &heigh_diffs.cycle.map(|r| r.clamp(-2, 2)),
                    );
                    MeshType::HexMapTile(normalized_diffs)
                }
                _ => mesh_type.clone(),
            })
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

        PbrBundle {
            mesh,
            material,
            transform,
            ..Default::default()
        }
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
        ];

        for key in materials {
            materials_map.insert(key, debug_material.clone());
        }

        let entries: Vec<(MeshType, Mesh)> = vec![
            (MeshType::Player, Sphere::new(layout.size.x).into()),
            (MeshType::Debug, Sphere::new(layout.size.x / 3.0).into()),
        ];

        for (key, mesh) in entries {
            meshes_map.insert(key, meshes.add(mesh));
        }
        for height_diff in PRECOMPUTED_HEIGHT_DIFF {
            meshes_map.insert(
                MeshType::HexMapTile(LexigraphicalCycle::shiloah_minimal_rotation(&height_diff)),
                meshes.add(Hexagon3D::create_base(
                    layout.size.x,
                    layout.orientation.starting_angle,
                    &height_diff,
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
