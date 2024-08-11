use std::num::NonZero;

use bevy::{
    asset::load_internal_asset,
    pbr::wireframe::Wireframe,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::*,
        render_asset::RenderAssetUsages,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types, BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
            Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, BufferVec,
            CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, Maintain,
            MapMode, PipelineCache, ShaderImport, ShaderStages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::{Receiver, Sender};

use crate::screen::Screen;

const CHUNK_SIZE: u32 = 16;
const CHUNK_SUBDIVISIONS: u32 = 2;
const CHUNK_POINTS_AMOUNT: usize = 2 << CHUNK_SUBDIVISIONS as usize;

const TERRAIN_SHADER_PATH: &str = "shaders/compute_noise.wgsl";
const SHADER_UTILS_NOISE: Handle<Shader> =
    Handle::weak_from_u128(0x0e78511e_e522_4bc3_aaa8_7d94ab2adcc2);

#[repr(C)]
#[derive(Component, Copy, Clone, Debug, Reflect, ExtractComponent, bytemuck::NoUninit)]
struct Chunk {
    position: IVec2,
}
impl Chunk {
    fn new(position: IVec2) -> Self {
        Self { position }
    }

    fn from_translation(translation: Vec3) -> Self {
        let position = translation - CHUNK_SIZE as f32 / 2.0;
        if position.fract().length() > 0.0 {
            error!("Chunk position is fractional");
        }
        Self::new(position.xy().as_ivec2())
    }
}

fn spawn_map(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        StateScoped(Screen::Playing),
        Chunk::from_translation(Vec3::ZERO),
        Wireframe,
        PbrBundle {
            mesh: asset_server.add(create_subdivided_plane(
                CHUNK_SUBDIVISIONS,
                CHUNK_SIZE as f32,
            )),
            material: asset_server.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..Default::default()
            }),
            ..Default::default()
        },
    ));
}

fn create_subdivided_plane(subdivisions: u32, size: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let num_vertices_per_side = subdivisions + 1;
    let num_vertices = (num_vertices_per_side * num_vertices_per_side) as usize;
    let num_indices = (subdivisions * subdivisions * 6) as usize;

    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for y in 0..=subdivisions {
        for x in 0..=subdivisions {
            let fx = x as f32 / subdivisions as f32 * size;
            let fy = y as f32 / subdivisions as f32 * size;

            positions.push([fx - 0.5, 0.0, fy - 0.5]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([fx, fy]);
        }
    }

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let i = y * (subdivisions + 1) + x;
            indices.push(i);
            indices.push(i + subdivisions + 1);
            indices.push(i + 1);

            indices.push(i + 1);
            indices.push(i + subdivisions + 1);
            indices.push(i + subdivisions + 2);
        }
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 16;
/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u32>>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u32>>);

fn receive(receiver: Res<MainWorldReceiver>) {
    // We don't want to block the main world on this,
    // so we use try_recv which attempts to receive without blocking
    if let Ok(data) = receiver.try_recv() {
        println!("Received data from render world: {data:?}");
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(GpuReadbackPlugin)
        .add_systems(OnEnter(Screen::Playing), spawn_map)
        .add_systems(Update, receive);
}

struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<Chunk>::default());
    }

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

/// Holds the buffers that will be used to communicate between the cpu and gpu
#[derive(Resource)]
struct Buffers {
    /// The buffer that will be used by the compute shader
    ///
    /// In this example, we want to write a `Vec<u32>` to a `Buffer`. `BufferVec` is a wrapper around a `Buffer`
    /// that will make sure the data is correctly aligned for the gpu and will simplify uploading the data to the gpu.
    gpu_buffer: BufferVec<u32>,
    /// The buffer that will be read on the cpu.
    /// The `gpu_buffer` will be copied to this buffer every frame
    cpu_buffer: Buffer,
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Create the buffer that will be accessed by the gpu
        let mut gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        for _ in 0..BUFFER_LEN {
            // Init the buffer with zeroes
            gpu_buffer.push(0);
        }
        // Write the buffer so the data is accessible on the gpu
        gpu_buffer.write_buffer(render_device, render_queue);

        // For portability reasons, WebGPU draws a distinction between memory that is
        // accessible by the CPU and memory that is accessible by the GPU. Only
        // buffers accessible by the CPU can be mapped and accessed by the CPU and
        // only buffers visible to the GPU can be used in shaders. In order to get
        // data from the GPU, we need to use `CommandEncoder::copy_buffer_to_buffer` to
        // copy the buffer modified by the GPU into a mappable, CPU-accessible buffer
        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (BUFFER_LEN * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gpu_buffer,
            cpu_buffer,
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
                .gpu_buffer
                .binding()
                // We already did it when creating the buffer so this should never happen
                .expect("Buffer should have already been uploaded to the gpu"),
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
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                binding_types::storage_buffer::<Vec<u32>>(false),
            ),
        );
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
    let buffer_slice = buffers.cpu_buffer.slice(..);

    // Now things get complicated. WebGPU, for safety reasons, only allows either the GPU
    // or CPU to access a buffer's contents at a time. We need to "map" the buffer which means
    // flipping ownership of the buffer over to the CPU and making access legal. We do this
    // with `BufferSlice::map_async`.
    //
    // The problem is that map_async is not an async function so we can't await it. What
    // we need to do instead is pass in a closure that will be executed when the slice is
    // either mapped or the mapping has failed.
    //
    // The problem with this is that we don't have a reliable way to wait in the main
    // code for the buffer to be mapped and even worse, calling get_mapped_range or
    // get_mapped_range_mut prematurely will cause a panic, not return an error.
    //
    // Using channels solves this as awaiting the receiving of a message from
    // the passed closure will force the outside code to wait. It also doesn't hurt
    // if the closure finishes before the outside code catches up as the message is
    // buffered and receiving will just pick that up.
    //
    // It may also be worth noting that although on native, the usage of asynchronous
    // channels is wholly unnecessary, for the sake of portability to Wasm
    // we'll use async channels that work on both native and Wasm.

    let (s, r) = crossbeam_channel::unbounded::<()>();

    // Maps the buffer so it can be read on the cpu
    buffer_slice.map_async(MapMode::Read, move |r| match r {
        // This will execute once the gpu is ready, so after the call to poll()
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    // In order for the mapping to be completed, one of three things must happen.
    // One of those can be calling `Device::poll`. This isn't necessary on the web as devices
    // are polled automatically but natively, we need to make sure this happens manually.
    // `Maintain::Wait` will cause the thread to wait on native but not on WebGpu.

    // This blocks until the gpu is done executing everything
    render_device.poll(Maintain::wait()).panic_on_timeout();

    // This blocks until the buffer is mapped
    r.recv().expect("Failed to receive the map_async message");

    {
        let buffer_view = buffer_slice.get_mapped_range();
        let data = buffer_view
            .chunks(std::mem::size_of::<u32>())
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("should be a u32")))
            .collect::<Vec<u32>>();
        sender
            .send(data)
            .expect("Failed to send data to main world");
    }

    // We need to make sure all `BufferView`'s are dropped before we do what we're about
    // to do.
    // Unmap so that we can copy to the staging buffer in the next iteration.
    buffers.cpu_buffer.unmap();
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
        render_context.command_encoder().copy_buffer_to_buffer(
            buffers
                .gpu_buffer
                .buffer()
                .expect("Buffer should have already been uploaded to the gpu"),
            0,
            &buffers.cpu_buffer,
            0,
            (BUFFER_LEN * std::mem::size_of::<u32>()) as u64,
        );

        Ok(())
    }
}
