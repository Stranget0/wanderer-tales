use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::{
    components::*,
    map::renderer::components::{MaterialType, MeshType},
};

pub trait RenderMapApi {
    fn get_render_item(&self, source_entity: &Entity) -> Option<&Entity>;
    fn remove_render_item(&mut self, source_entity: &Entity) -> Option<Entity>;
    fn link_source_item(
        &mut self,
        source_entity: &Entity,
        render_entity: &Entity,
    ) -> Option<Entity>;
}

pub trait CreateRenderBundles<T: Bundle> {
    fn create_render_bundle(
        &self,
        pos: &Vec3,
        rotation: &Rotation,
        material_type: &MaterialType,
        mesh_type: &MeshType,
    ) -> T;
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
}
