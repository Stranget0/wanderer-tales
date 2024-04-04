use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::data_source_layer::utils::HexLayout;
use crate::gameplay::{
    data_source_layer::map::components::Rotation,
    renderer::components::{MaterialType, MeshType},
};
pub trait RenderMapApi {
    fn get_render_item(&self, source_entity: &Entity) -> Option<&Entity>;
    fn remove_render_item(&mut self, source_entity: &Entity) -> Option<Entity>;
    fn link_source_item(
        &mut self,
        source_entity: &Entity,
        render_entity: &Entity,
    ) -> Option<Entity>;
    fn count(&self) -> usize;
}

pub trait SpawnRenderBundle {
    fn spawn_render_item(
        &mut self,
        commands: &mut Commands,
        source_entity: &Entity,
        pos: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,
        layout: (&Entity, &HexLayout),
        asset_server: &Res<AssetServer>,
    );
}

#[derive(Debug, Default)]
pub struct RenderMap(HashMap<u32, Entity>);

impl RenderMapApi for RenderMap {
    fn get_render_item(&self, source_item: &Entity) -> Option<&Entity> {
        self.0.get(&source_item.index())
    }

    fn remove_render_item(&mut self, source_entity: &Entity) -> Option<Entity> {
        self.0.remove(&source_entity.index())
    }

    fn link_source_item(
        &mut self,
        source_entity: &Entity,
        render_entity: &Entity,
    ) -> Option<Entity> {
        self.0.insert(source_entity.index(), *render_entity)
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}
