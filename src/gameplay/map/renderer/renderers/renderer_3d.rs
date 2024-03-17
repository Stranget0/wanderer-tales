use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::map::{
    renderer::{
        components::{MaterialType, MeshType},
        debug::uv_debug_texture,
    },
    utils::hex_layout::HexLayout,
};

use super::{
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

// impl CreateMapRenderBundle<PbrBundle> for Renderer3D {
//     fn create_map_render_bundle(
//         &self,
//         layout: &HexLayout,
//         pos: &HexPosition,
//         biome: &Biome,
//         height: &Height,
//     ) -> PbrBundle {
//         let transform = Self::get_hex_transform(layout, pos, height);
//         let material = self.get_hex_material(height, biome);
//         let mesh = self
//             .meshes_map
//             .get(&MeshType::HexMapTile)
//             .expect("Failed getting hex 2d mesh");

//         PbrBundle {
//             mesh: mesh.clone(),
//             material: material.clone(),
//             transform,
//             ..Default::default()
//         }
//     }
// }

// impl CreateCharacterRenderBundle<PbrBundle> for Renderer3D {
//     fn create_character_render_bundle(
//         &self,
//         pos: &Vec2,
//         source_entity: Entity,
//         material_key: MaterialType,
//         position: HexPositionFractional,
//     ) -> PbrBundle {
//         let mesh_handle = self
//             .meshes_map
//             .get(&MeshType::Player)
//             .expect("Player mesh not found");

//         let material_handle = self
//             .materials_map
//             .get(&material_key)
//             .unwrap_or_else(|| panic!("could not get {} material", material_key));

//         PbrBundle {
//             mesh: mesh_handle.clone(),
//             material: material_handle.clone(),
//             transform: Transform::from_xyz(pos.x, pos.y, 256.0),
//             ..default()
//         }
//     }
// }

impl CreateRenderBundle<PbrBundle> for Renderer3D {
    fn create_render_bundle(
        &self,
        pos: &Vec3,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> PbrBundle {
        let transform = Transform::from_xyz(pos.x, pos.y, pos.z);

        let material = self
            .materials_map
            .get(material_type)
            .unwrap_or_else(|| {
                self.materials_map
                    .get(&MaterialType::Debug)
                    .expect("Could not get debug material")
            })
            .clone();

        let mesh = self
            .meshes_map
            .get(mesh_type)
            .unwrap_or_else(|| {
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

        let entries: [(MeshType, Mesh); 2] = [
            (
                MeshType::HexMapTile,
                Hexagon3D::create_base(
                    layout.size.x,
                    layout.orientation.starting_angle,
                    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                ),
            ),
            (MeshType::Player, Sphere::new(layout.size.x).into()),
        ];

        for (key, mesh) in entries {
            meshes_map.insert(key, meshes.add(mesh));
        }

        Self {
            renders_map: RenderMap::default(),
            materials_map,
            meshes_map,
        }
    }
}
