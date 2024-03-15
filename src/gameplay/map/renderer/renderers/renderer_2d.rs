use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::HashMap,
};

use crate::gameplay::{
    map::{
        renderer::{events::RenderCharacterEvent, utils::MaterialKey},
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::FractionalHexVector,
        },
    },
    player::components::HexPosition,
};

use super::traits::{CreateCharacterRenderBundle, CreateMapRenderBundle, RenderMap, RenderMapApi};

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum MeshKey2d {
    Hex,
    Character,
}

#[derive(Component)]
pub struct Renderer2D {
    renders_map: RenderMap,
    pub materials_map: HashMap<MaterialKey, Handle<ColorMaterial>>,
    pub meshes_map: HashMap<MeshKey2d, Mesh2dHandle>,
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

impl CreateMapRenderBundle<MaterialMesh2dBundle<ColorMaterial>> for Renderer2D {
    fn create_map_render_bundle(
        &self,
        layout: &HexLayout,
        pos: &HexPosition,
        biome: &Biome,
        height: &Height,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        let pos = layout.hex_to_pixel(&FractionalHexVector::from(&pos.0));
        let transform = Transform::from_xyz(pos.x, pos.y, 0.0);
        let material_key = height.get_material();

        let material = self
            .materials_map
            .get(&material_key)
            .unwrap_or_else(|| panic!("failed getting {material_key} material"))
            .clone();

        let mesh = self
            .meshes_map
            .get(&MeshKey2d::Hex)
            .expect("Failed getting hex 2d mesh");

        MaterialMesh2dBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform,
            ..Default::default()
        }
    }
}

impl CreateCharacterRenderBundle<MaterialMesh2dBundle<ColorMaterial>> for Renderer2D {
    fn create_character_render_bundle(
        &self,
        pos: &Vec2,
        event: &RenderCharacterEvent,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        let mesh_handle = self
            .meshes_map
            .get(&MeshKey2d::Character)
            .expect("Player mesh not found");

        let material_handle = self
            .materials_map
            .get(&event.material_key)
            .unwrap_or_else(|| panic!("could not get {} material", event.material_key));

        MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(pos.x, pos.y, 2.0),
            ..default()
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
            (MaterialKey::Beach, Color::hex("#e1d76a")),
            (MaterialKey::Grass, Color::hex("#36b90b")),
            (MaterialKey::Forest, Color::hex("#054303")),
            (MaterialKey::Mountain, Color::hex("#302c2a")),
            (MaterialKey::Water, Color::hex("#0E499A")),
            (MaterialKey::Player, Color::hex("#f7f1d8")),
            (MaterialKey::Debug, Color::hex("#ea00ff")),
        ];

        for (key, color) in colors {
            let material_handle = materials.add(color.unwrap());
            materials_map.insert(key, material_handle);
        }

        let entries: [(MeshKey2d, Mesh); 2] = [
            (MeshKey2d::Hex, RegularPolygon::new(layout.size.x, 6).into()),
            (MeshKey2d::Character, Circle::new(3.0).into()),
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