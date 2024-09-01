use super::prelude::*;
use bevy::{
    prelude::*,
    render::{
        render_graph,
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::RenderContext,
    },
};
use std::marker::PhantomData;

pub trait RenderBurritoPassTrait<B, BG, P> {
    fn workgroup_size(&self, _world: &World) -> [u32; 3] {
        [1, 1, 1]
    }
    fn pipeline_key(&self, world: &World) -> &P;
    fn bind_group_key(&self, world: &World) -> &BG;
}

pub trait RenderBurritoNodeTrait<B, BG, P, Pass: RenderBurritoPassTrait<B, BG, P>> {
    fn passes(&self) -> &[Pass];
    fn label(&self) -> &str;
    fn staging_buffers(&self, world: &World) -> &[B];
    fn should_run(&self, _world: &World) -> bool {
        true
    }
}

pub struct NodeBurrito<
    N: RenderBurritoNodeTrait<B, BG, P, Pass>,
    Pass: RenderBurritoPassTrait<B, BG, P>,
    B,
    BL,
    BG,
    P,
> {
    node: N,
    _phantom: PhantomData<(B, BL, BG, P, Pass)>,
}

impl<N, Pass, B, BL, BG, P> NodeBurrito<N, Pass, B, BL, BG, P>
where
    N: RenderBurritoNodeTrait<B, BG, P, Pass> + Sync + Send + 'static,
    Pass: RenderBurritoPassTrait<B, BG, P> + Sync + Send + 'static,
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
    pub fn new(node: N) -> Self {
        Self {
            node,
            _phantom: PhantomData,
        }
    }
}

impl<N, Pass, B, BL, BG, P> render_graph::Node for NodeBurrito<N, Pass, B, BL, BG, P>
where
    N: RenderBurritoNodeTrait<B, BG, P, Pass> + Sync + Send + 'static,
    Pass: RenderBurritoPassTrait<B, BG, P> + Sync + Send + 'static,
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
    fn update(&mut self, world: &mut World) {
        if !self.node.should_run(world) {
            return;
        }
        for buffer in self.node.staging_buffers(world) {
            let wgsl = &mut world.resource_mut::<WgslRenderBurrito<B, BL, BG, P>>();
            let Some(buffer) = wgsl.get_buffer_readback_mut(buffer) else {
                continue;
            };
            buffer.mark_changed();
        }
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.node.should_run(world) {
            return Ok(());
        }

        for node_pass in self.node.passes() {
            let wgsl = world.resource::<WgslRenderBurrito<B, BL, BG, P>>();
            let pipeline_cache = world.resource::<PipelineCache>();
            let Some(pipeline) = wgsl
                .get_pipeline(node_pass.pipeline_key(world))
                .and_then(|p| pipeline_cache.get_compute_pipeline(*p))
            else {
                return Ok(());
            };

            let Some(bind_group) = wgsl.get_bind_group(&node_pass.bind_group_key(world)) else {
                return Ok(());
            };

            let mut render_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some(self.node.label()),
                        ..default()
                    });

            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_pipeline(pipeline);

            let [x, y, z] = node_pass.workgroup_size(world);
            render_pass.dispatch_workgroups(x, y, z);
        }

        // Copy the gpu accessible buffer to the cpu accessible buffer
        let wgsl = world.resource::<WgslRenderBurrito<B, BL, BG, P>>();
        info!("copying buffers");
        let encoder = render_context.command_encoder();
        for buffer in self.node.staging_buffers(world) {
            wgsl.copy_to_readback_buffer(encoder, &buffer);
        }
        Ok(())
    }
}
