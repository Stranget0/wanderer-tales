use crate::utils::{insert_burrito_channel, WgslBurritoPlugin, WgslMainBurrito, WgslRenderBurrito};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderBufferKey {
    MapPointPositions,
    MapPointData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderLayoutKey {
    MapLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderBindGroupKey {
    MapBindGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPipelineKey {
    MapPipeline,
}

pub type RenderStatePlugin =
    WgslBurritoPlugin<RenderBufferKey, RenderLayoutKey, RenderBindGroupKey, RenderPipelineKey>;

pub type RenderStateMain = WgslMainBurrito<RenderBufferKey>;

pub type RenderStateRender =
    WgslRenderBurrito<RenderBufferKey, RenderLayoutKey, RenderBindGroupKey, RenderPipelineKey>;

pub fn insert_readback_channel(app: &mut App, buffer_key: RenderBufferKey) {
    insert_burrito_channel::<RenderBufferKey, RenderLayoutKey, RenderBindGroupKey, RenderPipelineKey>(
        app, buffer_key,
    );
}
