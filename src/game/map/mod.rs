use std::num::NonZero;

use bevy::{
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
            MapMode, PipelineCache, ShaderStages,
        },
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};

use itertools::Itertools;

use crate::screen::Screen;

const CHUNK_SIZE: u32 = 16;
const CHUNK_SUBDIVISIONS: u32 = 2;
const CHUNK_POINTS_AMOUNT: usize = 2 << CHUNK_SUBDIVISIONS as usize;

const TERRAIN_SHADER_PATH: &str = "shaders/compute_noise.wgsl";

#[repr(C)]
#[derive(Component, Copy, Clone, Debug, Reflect, ExtractComponent, bytemuck::NoUninit)]
struct Chunk {
    position: IVec2,
}

struct ChunkPointData {
    height: f32,
}

#[derive(Resource)]
struct BindGroups(pub BindGroup);

#[derive(Resource)]
struct Buffers {
    pub chunk_size_uniform: Buffer,
    pub chunk_subdivisions_uniform: Buffer,
    pub input_buffer: Buffer,
    pub output_buffer: Buffer,
    pub cpu_buffer: Buffer,
    pub chunks_len: u32,
    size_output: u64,
}

#[derive(Resource)]
struct ChunkPipeline {
    bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Playing), spawn_map)
            .add_plugins(ExtractComponentPlugin::<Chunk>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<ChunkPipeline>().add_systems(
            Render,
            (
                prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
                map_and_read_buffer
                    .after(RenderSet::PrepareBindGroups)
                    .run_if(resource_exists::<Buffers>),
            ),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ChunkComputeLabel, ChunkComputeNode::default());
    }
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

impl FromWorld for ChunkPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let shader = world.load_asset(TERRAIN_SHADER_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();

        let bind_group_layout = device.create_bind_group_layout(
            "chunk_bind_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    binding_types::uniform_buffer::<u32>(false),
                    binding_types::uniform_buffer::<u32>(false),
                    binding_types::storage_buffer_read_only::<Vec<IVec2>>(false),
                    binding_types::storage_buffer::<Vec<f32>>(false),
                ),
            ),
        );

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "main".into(),
        });

        Self {
            bind_group_layout,
            update_pipeline,
        }
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ChunkPipeline>,
    device: Res<RenderDevice>,
    chunks: Query<&Chunk>,
) {
    let chunks = chunks.iter().copied().collect_vec();
    let chunks_len = chunks.len();
    if chunks_len == 0 {
        commands.remove_resource::<Buffers>();
        return;
    }

    let size_output = (chunks_len * size_of::<f32>()) as u64;

    let chunk_size_uniform = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("chunk_size_uniform"),
        contents: bytemuck::bytes_of(&CHUNK_SIZE),
        usage: BufferUsages::UNIFORM,
    });

    let chunk_subdivisions_uniform = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("chunk_subdivisions_uniform"),
        contents: bytemuck::bytes_of(&CHUNK_SUBDIVISIONS),
        usage: BufferUsages::UNIFORM,
    });

    let input_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("chunk_buffer_input"),
        contents: bytemuck::cast_slice(chunks.as_slice()),
        usage: BufferUsages::STORAGE,
    });

    let output_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("chunk_buffer_output"),
        size: size_output,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let cpu_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("chunk_buffer_cpu"),
        size: size_output,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(
        None,
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((
            chunk_size_uniform.as_entire_binding(),
            chunk_subdivisions_uniform.as_entire_binding(),
            input_buffer.as_entire_binding(),
            output_buffer.as_entire_buffer_binding(),
        )),
    );

    commands.insert_resource(BindGroups(bind_group));
    commands.insert_resource(Buffers {
        chunk_size_uniform,
        chunk_subdivisions_uniform,
        input_buffer,
        output_buffer,
        chunks_len: chunks_len as u32,
        size_output,
        cpu_buffer,
    });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ChunkComputeLabel;

#[derive(Default)]
struct ChunkComputeNode {}

impl render_graph::Node for ChunkComputeNode {
    fn run<'w>(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let chunk_pipeline = world.resource::<ChunkPipeline>();
        let Some(buffers) = world.get_resource::<Buffers>() else {
            return Ok(());
        };
        let Some(bind_group) = world.get_resource::<BindGroups>() else {
            return Ok(());
        };

        if let Some(pipeline) = pipeline_cache.get_compute_pipeline(chunk_pipeline.update_pipeline)
        {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Chunk compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(buffers.chunks_len, 1, 1);
        }

        render_context.command_encoder().copy_buffer_to_buffer(
            &buffers.output_buffer,
            0,
            &buffers.cpu_buffer,
            0,
            buffers.size_output,
        );

        Ok(())
    }
}

fn map_and_read_buffer(render_device: Res<RenderDevice>, buffers: Res<Buffers>) {
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
        Err(err) => {
            panic!("Failed to map buffer {:?}", err)
        }
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
            .chunks(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("should be a f32")))
            .collect::<Vec<f32>>();

        info!("Result: {:?}", data);
        // sender
        //     .send(data)
        //     .expect("Failed to send data to main world");
    }

    // We need to make sure all `BufferView`'s are dropped before we do what we're about
    // to do.
    // Unmap so that we can copy to the staging buffer in the next iteration.
    buffers.cpu_buffer.unmap();
}
