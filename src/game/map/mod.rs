//! A very simple compute shader that updates a gpu buffer.
//! That buffer is then copied to the cpu and sent to the main world.
//!
//! This example is not meant to teach compute shaders.
//! It is only meant to explain how to read a gpu buffer on the cpu and then use it in the main world.
//!
//! The code is based on this wgpu example:
//! <https://github.com/gfx-rs/wgpu/blob/fb305b85f692f3fbbd9509b648dfbc97072f7465/examples/src/repeated_compute/mod.rs>

use crate::utils::*;
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 16;

// We need a plugin to organize all the systems and render node required for this example
pub struct GpuReadbackPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum BufferLabels {
    TestBuffer,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum LayoutLabels {
    TestLayout,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum BindGroupLabels {
    TestBindGroup,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum PipelineLabels {
    TestPipeline,
}

impl std::fmt::Display for BufferLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "buffer")
    }
}
impl std::fmt::Display for LayoutLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "layout")
    }
}
impl std::fmt::Display for BindGroupLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bind-group")
    }
}
impl std::fmt::Display for PipelineLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pipeline")
    }
}

type TestRenderBurrito =
    WgslRenderBurrito<BufferLabels, LayoutLabels, BindGroupLabels, PipelineLabels>;

type TestMainBurrito = WgslMainBurrito<BufferLabels>;

type TestBurritoPlugin =
    WgslBurritoPlugin<BufferLabels, LayoutLabels, BindGroupLabels, PipelineLabels>;

type BufferType = u32;
type BufferVecType = Vec<BufferType>;

impl Plugin for GpuReadbackPlugin {
    fn build(&self, _app: &mut App) {}

    // The render device is only accessible inside finish().
    // So we need to initialize render resources here.
    fn finish(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(TestMainBurrito::new())
            .add_systems(Update, receive);

        app.world_mut()
            .resource_mut::<TestMainBurrito>()
            .insert_receiver(BufferLabels::TestBuffer, receiver);

        let render_app = &mut app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(TestRenderBurrito::new())
            .add_systems(
                Render,
                (
                    setup_pipeline.before(RenderSet::PrepareBindGroups),
                    prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
                    map_and_read_buffer::<
                        BufferLabels,
                        LayoutLabels,
                        BindGroupLabels,
                        PipelineLabels,
                    >
                        .after(RenderSet::Render),
                ),
            );

        render_app
            .world_mut()
            .resource_mut::<TestRenderBurrito>()
            .insert_sender(BufferLabels::TestBuffer, sender);

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

fn receive(wgsl_main: Res<TestMainBurrito>) {
    let Some(data) = wgsl_main.try_receive_vec::<BufferType>(BufferLabels::TestBuffer) else {
        return;
    };

    info!("received data: {data:?}");
}

fn setup_pipeline(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    mut wgsl: ResMut<TestRenderBurrito>,
    asset_server: Res<AssetServer>,
    mut has_run: Local<bool>,
) {
    if *has_run {
        return;
    }
    *has_run = true;

    info!("setup pipeline");
    wgsl.start_create_layout(
        LayoutLabels::TestLayout,
        ShaderStages::COMPUTE,
        render_device.as_ref(),
    )
    .storage_slot::<BufferVecType>()
    .build();

    wgsl.start_create_pipeline(
        PipelineLabels::TestPipeline,
        pipeline_cache.as_ref(),
        asset_server.load(SHADER_ASSET_PATH),
        "main",
    )
    .with_layouts(&[LayoutLabels::TestLayout])
    .build();
}

fn prepare_bind_group(render_device: Res<RenderDevice>, mut wgsl: ResMut<TestRenderBurrito>) {
    wgsl.start_create_buffers(render_device.as_ref())
        .create_once_empty_storage_readable(
            BufferLabels::TestBuffer,
            (BUFFER_LEN * std::mem::size_of::<u32>()) as u64,
        );

    wgsl.create_once_bind_group(
        BindGroupLabels::TestBindGroup,
        render_device.as_ref(),
        LayoutLabels::TestLayout,
        &[BufferLabels::TestBuffer],
    );
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

/// The node that will execute the compute shader
#[derive(Default)]
struct ComputeNode {}
impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let wgsl = world.resource::<TestRenderBurrito>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = wgsl.get_pipeline(&PipelineLabels::TestPipeline);
        let bind_group = wgsl.get_bind_group(&BindGroupLabels::TestBindGroup);

        if let Some(init_pipeline) = pipeline.and_then(|p| pipeline_cache.get_compute_pipeline(*p))
        {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, bind_group.expect("bind group not found"), &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(BUFFER_LEN as u32, 1, 1);
        }

        // Copy the gpu accessible buffer to the cpu accessible buffer
        wgsl.copy_to_readback_buffer(&BufferLabels::TestBuffer, render_context.command_encoder());

        Ok(())
    }
}
