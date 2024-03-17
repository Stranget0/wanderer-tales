use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::{
    map::{
        renderer::{
            components::RenderType, debug::uv_debug_texture, events::RenderCharacterEvent,
            utils::MaterialKey,
        },
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::FractionalHexVector,
        },
    },
    player::components::HexPosition,
};

use super::{
    meshes::Hexagon3D,
    traits::{CreateCharacterRenderBundle, CreateMapRenderBundle, RenderMap, RenderMapApi},
};

#[derive(Component)]
pub struct Renderer3D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialKey, Handle<StandardMaterial>>,
    pub meshes_map: HashMap<RenderType, Handle<Mesh>>,
}

impl Renderer3D {
    fn get_hex_transform(layout: &HexLayout, pos: &HexPosition, height: &Height) -> Transform {
        let pos = layout.hex_to_pixel(&FractionalHexVector::from(&pos.0));

        Transform::from_xyz(pos.x, pos.y, height.get_height().into())
    }
    fn get_hex_material(&self, height: &Height, biome: &Biome) -> Handle<StandardMaterial> {
        {
            let material_key = height.get_material();

            self.materials_map
                .get(&material_key)
                .unwrap_or_else(|| panic!("failed getting {material_key} material"))
                .clone()
        }
    }
    fn get_hex_mesh(&self) -> Handle<Mesh> {
        self.meshes_map
            .get(&RenderType::HexMapTile)
            .expect("Could not get hex mesh")
            .clone()
    }
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

impl CreateMapRenderBundle<PbrBundle> for Renderer3D {
    fn create_map_render_bundle(
        &self,
        layout: &HexLayout,
        pos: &HexPosition,
        biome: &Biome,
        height: &Height,
    ) -> PbrBundle {
        let transform = Self::get_hex_transform(layout, pos, height);
        let material = self.get_hex_material(height, biome);
        let mesh = self
            .meshes_map
            .get(&RenderType::HexMapTile)
            .expect("Failed getting hex 2d mesh");

        PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform,
            ..Default::default()
        }
    }
}

impl CreateCharacterRenderBundle<PbrBundle> for Renderer3D {
    fn create_character_render_bundle(
        &self,
        pos: &Vec2,
        event: &RenderCharacterEvent,
    ) -> PbrBundle {
        let mesh_handle = self
            .meshes_map
            .get(&RenderType::Player)
            .expect("Player mesh not found");

        let material_handle = self
            .materials_map
            .get(&event.material_key)
            .unwrap_or_else(|| panic!("could not get {} material", event.material_key));

        PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(pos.x, pos.y, 256.0),
            ..default()
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
            MaterialKey::Beach,
            MaterialKey::Grass,
            MaterialKey::Forest,
            MaterialKey::Mountain,
            MaterialKey::Water,
            MaterialKey::Player,
        ];

        for key in materials {
            materials_map.insert(key, debug_material.clone());
        }

        let entries: [(RenderType, Mesh); 2] = [
            (
                RenderType::HexMapTile,
                Hexagon3D::create_mesh(layout.size.x, layout.orientation.starting_angle),
            ),
            (RenderType::Player, Sphere::new(layout.size.x).into()),
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
