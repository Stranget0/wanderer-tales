use bevy::app::Plugin;

use self::{renderer_2d::Renderer2DPlugin, renderer_3d_tiles::Renderer3DPlugin};

pub mod components;
pub mod debug;
pub mod events;
pub mod renderer_2d;
pub mod renderer_3d_tiles;
pub mod state;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((Renderer3DPlugin /*Renderer3DPlugin*/,));
    }
}
