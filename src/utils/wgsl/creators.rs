use bevy::render::{
    render_resource::{
        encase::{internal::WriteInto, StorageBuffer, UniformBuffer},
        BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer, BufferDescriptor,
        BufferInitDescriptor, BufferUsages, ShaderType,
    },
    renderer::RenderDevice,
};

pub fn storage_empty_rw<K>(device: &RenderDevice, name: &K, size: u64) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some(format!("{}--storage-rw", name).as_str()),
        size,
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    buffer
}

pub fn storage_empty<K>(device: &RenderDevice, name: &K, size: u64) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    device.create_buffer(&BufferDescriptor {
        label: Some(format!("{}--storage-r", name).as_str()),
        size,
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

pub fn uniform_empty<K, T: WriteInto + ShaderType>(
    device: &RenderDevice,
    name: &K,
    size: u64,
) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    T::assert_uniform_compat();

    device.create_buffer(&BufferDescriptor {
        label: Some(format!("{}--uniform-r", name).as_str()),
        size,
        usage: BufferUsages::COPY_SRC | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    })
}

pub fn storage_buffer_rw<K, T: WriteInto + ShaderType>(
    device: &RenderDevice,
    name: &K,
    payload: &T,
) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    let mut buffer = StorageBuffer::new(Vec::new());
    buffer.write::<T>(payload).unwrap();

    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some(format!("{}--storage-rw", name).as_str()),
        contents: buffer.as_ref(),
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
    })
}

pub fn storage_buffer<K, T: WriteInto + ShaderType>(
    device: &RenderDevice,
    name: &K,
    payload: &T,
) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    let mut buffer = StorageBuffer::new(Vec::new());
    buffer.write::<T>(payload).unwrap();

    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some(format!("{}--storage-r", name).as_str()),
        contents: buffer.as_ref(),
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
    })
}

pub fn uniform_buffer<K, T: WriteInto + ShaderType>(
    device: &RenderDevice,
    name: &K,
    payload: &T,
) -> Buffer
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    T::assert_uniform_compat();

    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write::<T>(payload).unwrap();

    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some(format!("{}--uniform-r", name).as_str()),
        contents: buffer.as_ref(),
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
    })
}

pub fn cpu_buffer<K: std::fmt::Display>(device: &RenderDevice, name: &K, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(format!("{}--cpu-buffer", name).as_str()),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_bind_group_layout<N>(
    device: &&RenderDevice,
    name: N,
    entries: &[BindGroupLayoutEntry],
) -> BindGroupLayout
where
    N: std::fmt::Display,
{
    device.create_bind_group_layout(Some(format!("{}--layout", name).as_str()), entries)
}

pub fn create_bind_group<N: std::fmt::Display>(
    device: &RenderDevice,
    bind_group_name: N,
    layout: &BindGroupLayout,
    bind_group_entries: Vec<BindGroupEntry>,
) -> BindGroup {
    device.create_bind_group(
        Some(format!("{}--bind-group", bind_group_name).as_str()),
        layout,
        bind_group_entries.as_slice(),
    )
}
