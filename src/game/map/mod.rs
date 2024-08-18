//! A very simple compute shader that updates a gpu buffer.
//! That buffer is then copied to the cpu and sent to the main world.
//!
//! This example is not meant to teach compute shaders.
//! It is only meant to explain how to read a gpu buffer on the cpu and then use it in the main world.
//!
//! The code is based on this wgpu example:
//! <https://github.com/gfx-rs/wgpu/blob/fb305b85f692f3fbbd9509b648dfbc97072f7465/examples/src/repeated_compute/mod.rs>

use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
    utils::hashbrown::HashMap,
};
use crossbeam_channel::{Receiver, Sender};

use crate::utils::{BindLayoutBuilder, BufferBuilder, ReadbackBuffer};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 16;

// To communicate between the main world and the render world we need a channel.
// Since the main world and render world run in parallel, there will always be a frame of latency
// between the data sent from the render world and the data received in the main world
//
// frame n => render world sends data through the channel at the end of the frame
// frame n + 1 => main world receives the data

/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u32>>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u32>>);

pub fn plugin(app: &mut App) {
    app.add_plugins(GpuReadbackPlugin)
        .add_systems(Update, receive);
}

/// This system will poll the channel and try to get the data sent from the render world
fn receive(receiver: Res<MainWorldReceiver>) {
    // We don't want to block the main world on this,
    // so we use try_recv which attempts to receive without blocking
    if let Ok(data) = receiver.try_recv() {
        println!("Received data from render world: {data:?}");
    }
}

// We need a plugin to organize all the systems and render node required for this example
struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, _app: &mut App) {}

    // The render device is only accessible inside finish().
    // So we need to initialize render resources here.
    fn finish(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(RenderWorldSender(s))
            .init_resource::<ComputePipeline>()
            .init_resource::<Buffers>()
            .add_systems(
                Render,
                (
                    prepare_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        // We don't need to recreate the bind group every frame
                        .run_if(not(resource_exists::<GpuBufferBindGroup>)),
                    // We need to run it after the render graph is done
                    // because this needs to happen after submit()
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum BufferLabels {
    Buffer,
}
impl std::fmt::Display for BufferLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "buffer")
    }
}

/// Holds the buffers that will be used to communicate between the cpu and gpu
#[derive(Resource)]
struct Buffers {
    buffers: HashMap<BufferLabels, Buffer>,
    readback_buffers: HashMap<BufferLabels, ReadbackBuffer>,
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let (buffers, readback_buffers) = BufferBuilder::new(render_device)
            .create_empty_storage_readable(
                BufferLabels::Buffer,
                (BUFFER_LEN * std::mem::size_of::<u32>()) as u64,
            )
            .build();

        Self {
            buffers,
            readback_buffers,
        }
    }
}

#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::single(
            buffers
                .buffers
                .get(&BufferLabels::Buffer)
                .expect("Buffer not found")
                .as_entire_binding(),
        ),
    );
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = BindLayoutBuilder::new(render_device, "compute-layout", ShaderStages::COMPUTE)
            .create_storage::<Vec<u32>>()
            .build();

        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("GPU readback compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        });
        ComputePipeline { layout, pipeline }
    }
}

fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
    sender: Res<RenderWorldSender>,
) {
    // Finally time to get our data back from the gpu.
    // First we get a buffer slice which represents a chunk of the buffer (which we
    // can't access yet).
    // We want the whole thing so use unbounded range.
    let label = BufferLabels::Buffer;
    let buffer = &buffers
        .readback_buffers
        .get(&label)
        .expect("No buffer reader for {label}");

    buffer.poll_map_and_read(render_device.as_ref(), sender.as_ref());
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
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(BUFFER_LEN as u32, 1, 1);
        }

        // Copy the gpu accessible buffer to the cpu accessible buffer
        let buffers = world.resource::<Buffers>();
        let gpu_buffer = buffers.buffers.get(&BufferLabels::Buffer).unwrap();
        let cpu_buffer = &buffers
            .readback_buffers
            .get(&BufferLabels::Buffer)
            .unwrap()
            .buffer;

        render_context.command_encoder().copy_buffer_to_buffer(
            gpu_buffer,
            0,
            cpu_buffer,
            0,
            cpu_buffer.size(),
        );

        Ok(())
    }
}
