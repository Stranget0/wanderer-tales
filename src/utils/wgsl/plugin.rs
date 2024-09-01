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
        app.insert_resource(WgslMainBurrito::<B>::new())
            .sub_app_mut(RenderApp)
            .insert_resource(WgslRenderBurrito::<B, BL, BG, P>::new())
            .add_systems(
                Render,
                // We need to run it after the render graph is done
                // because this needs to happen after submit()
                map_and_read_buffer::<B, BL, BG, P>.after(RenderSet::Render),
            );
    }
}

pub fn insert_burrito_channel<B, BL, BG, P>(app: &mut App, buffer_key: B)
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
    let (sender, receiver) = crossbeam_channel::unbounded();
    app.world_mut()
        .resource_mut::<WgslMainBurrito<B>>()
        .insert_receiver(buffer_key.to_owned(), receiver);

    let render_app = &mut app.sub_app_mut(RenderApp);
    render_app
        .world_mut()
        .resource_mut::<WgslRenderBurrito<B, BL, BG, P>>()
        .insert_sender(buffer_key.to_owned(), sender);
}
