use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::gameplay::{
    map::{
        renderer::events::RenderCharacterEvent,
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
        },
    },
    player::components::HexPosition,
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

pub trait CreateMapRenderBundle<T: Bundle> {
    fn create_map_render_bundle(
        &self,
        layout: &HexLayout,
        pos: &HexPosition,
        biome: &Biome,
        height: &Height,
    ) -> T;
}

pub trait CreateCharacterRenderBundle<T: Bundle> {
    fn create_character_render_bundle(&self, pos: &Vec2, event: &RenderCharacterEvent) -> T;
}

#[derive(Debug)]
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

impl Default for RenderMap {
    fn default() -> Self {
        Self(Default::default())
    }
}
