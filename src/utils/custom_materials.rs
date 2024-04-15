use bevy::{pbr::MaterialExtension, prelude::*, render::render_resource::*};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct MyExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
}

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/world_aligned.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/world_aligned.wgsl".into()
    }
}
