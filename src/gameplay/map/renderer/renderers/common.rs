use bevy::{prelude::*, utils::hashbrown::HashMap};

use super::traits::RenderMapApi;
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
