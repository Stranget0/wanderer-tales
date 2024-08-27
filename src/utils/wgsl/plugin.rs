use bevy::prelude::*;
use bevy::render::Render;
use bevy::render::RenderApp;
use bevy::render::RenderSet;
use std::marker::PhantomData;

use super::resources::*;

pub struct WgslBurritoPlugin<B, BL, BG, P> {
    pub(crate) _phantom: PhantomData<(B, BL, BG, P)>,
}

impl<B, BL, BG, P> WgslBurritoPlugin<B, BL, BG, P> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<B, BL, BG, P> Default for WgslBurritoPlugin<B, BL, BG, P> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<B, BL, BG, P> Plugin for WgslBurritoPlugin<B, BL, BG, P>
where
    B: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Debug
        + ToOwned<Owned = B>
        + Send
        + Sync
        + 'static,
    BL: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Debug
        + ToOwned<Owned = BL>
        + Send
        + Sync
        + 'static,
    BG: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Debug
        + ToOwned<Owned = BG>
        + Send
        + Sync
        + 'static,
    P: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Debug
        + ToOwned<Owned = P>
        + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(WgslMainBurrito::<B>::new());
    }
    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .insert_resource(WgslRenderBurrito::<B, BL, BG, P>::new())
            .add_systems(
                Render,
                // We need to run it after the render graph is done
                // because this needs to happen after submit()
                map_and_read_buffer::<B, BL, BG, P>.after(RenderSet::Render),
            );
    }
}
