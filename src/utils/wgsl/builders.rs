use super::buffers_readback::*;
use super::creators;
use super::errors::BindGroupBuilderError;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::{
    render::{
        render_resource::{
            binding_types, encase::internal::WriteInto, BindGroup, BindGroupEntry,
            BindGroupLayoutEntry, BindGroupLayoutEntryBuilder, Buffer, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
    utils::hashbrown::HashMap,
};

pub fn buffer_builder<
    'a,
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
>(
    device: &'a RenderDevice,
) -> BufferBuilder<'a, K> {
    BufferBuilder::new(device)
}

pub struct BufferBuilder<
    'a,
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
> {
    device: &'a RenderDevice,
    buffer_map: HashMap<K, Buffer>,
    readback_buffer_map: HashMap<K, ReadbackBuffer>,
}

pub(crate) struct BufferMutateBuilder<
    'a,
    'b,
    'c,
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned,
> {
    device: &'a RenderDevice,
    buffer_map: &'b mut HashMap<K, Buffer>,
    readback_buffer_map: &'c mut HashMap<K, ReadbackBuffer>,
}

pub trait BufferBuilderExt<
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
>
{
    fn has_buffer(&self, name: &K) -> bool;
    fn insert_buffer(&mut self, name: K, buffer: Buffer);

    fn insert_readback_buffer(&mut self, name: K, buffer: Buffer);
    fn get_device(&self) -> &RenderDevice;

    fn create_uniform<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        T::assert_uniform_compat();
        let device = self.get_device();
        let buffer = creators::uniform_buffer(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    fn create_storage<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    fn create_storage_rw<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer_rw(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    fn create_storage_readable<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer_rw(device, &name, payload);
        let size = buffer.size();
        let readback_buffer = creators::cpu_buffer::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    fn create_empty_storage_readable(mut self, name: K, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty_rw(device, &name, size);
        let size = buffer.size();
        let readback_buffer = creators::cpu_buffer::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    fn create_empty_uniform<T: WriteInto + ShaderType>(mut self, name: K, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::uniform_empty::<K, T>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    fn create_empty_storage<T: WriteInto + ShaderType>(mut self, name: K, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    fn create_empty_storage_rw<T: WriteInto + ShaderType>(mut self, name: K, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty_rw::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }
}

impl<'a, 'b, 'c, K> BufferMutateBuilder<'a, 'b, 'c, K>
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
{
    pub fn new(
        device: &'a RenderDevice,
        buffer_map: &'b mut HashMap<K, Buffer>,
        readback_buffer_map: &'c mut HashMap<K, ReadbackBuffer>,
    ) -> Self {
        Self {
            device,
            buffer_map,
            readback_buffer_map,
        }
    }

    pub fn create_once_empty_storage_readable(self, name: K, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage_readable(name, size)
    }
    pub fn create_once_storage_readable<T: WriteInto + ShaderType>(
        self,
        name: K,
        payload: &T,
    ) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_storage_readable(name, payload)
    }
    pub fn create_once_empty_uniform<T: WriteInto + ShaderType>(self, name: K, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_uniform::<T>(name, size)
    }
    pub fn create_once_empty_storage<T: WriteInto + ShaderType>(self, name: K, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage::<T>(name, size)
    }
    pub fn create_once_empty_storage_rw<T: WriteInto + ShaderType>(
        self,
        name: K,
        size: u64,
    ) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage_rw::<T>(name, size)
    }
}

impl<'a, 'b, 'c, K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>>
    BufferBuilder<'a, K>
{
    pub fn new(device: &'a RenderDevice) -> Self {
        Self {
            device,
            buffer_map: HashMap::new(),
            readback_buffer_map: HashMap::new(),
        }
    }
}
impl<'a, 'b, 'c, K> BufferBuilderExt<K> for BufferBuilder<'a, K>
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
{
    fn has_buffer(&self, name: &K) -> bool {
        self.buffer_map.contains_key(name)
    }

    fn insert_buffer(&mut self, name: K, buffer: Buffer) {
        self.buffer_map.insert(name, buffer);
    }

    fn insert_readback_buffer(&mut self, name: K, buffer: Buffer) {
        self.readback_buffer_map
            .insert(name, ReadbackBuffer::new(buffer));
    }

    fn get_device(&self) -> &RenderDevice {
        self.device
    }
}

impl<'a, 'b, 'c, K> BufferBuilderExt<K> for BufferMutateBuilder<'a, 'b, 'c, K>
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
{
    fn insert_buffer(&mut self, name: K, buffer: Buffer) {
        self.buffer_map.insert(name, buffer);
    }

    fn insert_readback_buffer(&mut self, name: K, buffer: Buffer) {
        self.readback_buffer_map
            .insert(name, ReadbackBuffer::new(buffer));
    }

    fn has_buffer(&self, name: &K) -> bool {
        self.buffer_map.contains_key(name)
    }

    fn get_device(&self) -> &RenderDevice {
        self.device
    }
}

pub struct BindLayoutMutateBuilder<
    'a,
    'b,
    N: std::fmt::Display + PartialEq + Eq + std::hash::Hash + ToOwned<Owned = N>,
> {
    name: N,
    device: &'a RenderDevice,
    layout_map: &'b mut HashMap<N, BindGroupLayout>,
    entries: Vec<BindGroupLayoutEntry>,
    visibility: ShaderStages,
}

impl<'a, 'b, N> BindLayoutMutateBuilder<'a, 'b, N>
where
    N: std::fmt::Display + PartialEq + Eq + std::hash::Hash + ToOwned<Owned = N>,
{
    pub fn new(
        device: &'a RenderDevice,
        name: N,
        visibility: ShaderStages,
        layout_map: &'b mut HashMap<N, BindGroupLayout>,
    ) -> Self {
        Self {
            device,
            name,
            visibility,
            layout_map,
            entries: Vec::new(),
        }
    }

    fn push_binding(&mut self, binding: BindGroupLayoutEntryBuilder) {
        let binding_index = self.entries.len() as u32;

        self.entries
            .push(binding.build(binding_index, self.visibility));
    }

    pub fn uniform_slot<T: WriteInto + ShaderType>(mut self) -> Self {
        T::assert_uniform_compat();

        self.push_binding(binding_types::uniform_buffer::<T>(false));

        self
    }

    pub fn storage_slot<T: WriteInto + ShaderType>(mut self) -> Self {
        self.push_binding(binding_types::storage_buffer::<T>(false));

        self
    }

    pub fn build(self) {
        let device = &self.device;
        let entries = self.entries.as_slice();
        let name = self.name;

        let layout = creators::create_bind_group_layout(device, name.to_owned(), entries);

        self.layout_map.insert(name, layout);
    }
}

pub fn bind_group<
    N: std::fmt::Display,
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = K>,
>(
    bind_group_name: N,
    device: &RenderDevice,
    layout: &BindGroupLayout,
    buffers: &HashMap<K, Buffer>,
    binds: &[K],
) -> Result<BindGroup, BindGroupBuilderError> {
    let mut bind_group_entries = Vec::with_capacity(binds.len());
    for (index, name) in binds.iter().enumerate() {
        let buffer = match buffers.get(name) {
            Some(entry) => entry,
            None => return Err(BindGroupBuilderError::no_buffer_found(name.to_owned())),
        };

        bind_group_entries.push(BindGroupEntry {
            binding: index as u32,
            resource: buffer.as_entire_binding(),
        });
    }

    Ok(creators::create_bind_group(
        device,
        bind_group_name,
        layout,
        bind_group_entries,
    ))
}

pub struct PipelineMutateBuilder<
    'a,
    'b,
    'c,
    'd,
    K: std::fmt::Display + PartialEq + Eq + std::hash::Hash,
    BL: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
> {
    pipeline_cache: &'a PipelineCache,
    shader: Handle<Shader>,
    shader_defs: Vec<ShaderDefVal>,
    push_constant_ranges: Vec<PushConstantRange>,
    layouts_map: &'b HashMap<BL, BindGroupLayout>,
    entry_point: &'c str,
    pipelines_map: &'d mut HashMap<K, CachedComputePipelineId>,
    layouts: Vec<BindGroupLayout>,
    name: K,
}

impl<
        'a,
        'b,
        'c,
        'd,
        K: std::fmt::Display + PartialEq + Eq + std::hash::Hash,
        BL: PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    > PipelineMutateBuilder<'a, 'b, 'c, 'd, K, BL>
{
    pub fn new(
        name: K,
        pipeline_cache: &'a PipelineCache,
        shader: Handle<Shader>,
        layouts_map: &'b HashMap<BL, BindGroupLayout>,
        entry_point: &'c str,
        pipelines_map: &'d mut HashMap<K, CachedComputePipelineId>,
    ) -> Self {
        Self {
            pipeline_cache,
            shader,
            layouts_map,
            name,
            entry_point,
            pipelines_map,
            shader_defs: Vec::new(),
            push_constant_ranges: Vec::new(),
            layouts: Vec::new(),
        }
    }

    pub fn with_shader_defs(mut self, shader_defs: Vec<ShaderDefVal>) -> Self {
        self.shader_defs = shader_defs;

        self
    }

    pub fn with_layout(mut self, layout: &BL) -> Self {
        if let Some(layout) = self.layouts_map.get(layout) {
            self.layouts.push(layout.clone());
        } else {
            let error = BindGroupBuilderError::no_layout_found(layout);
            let name = &self.name;
            error!("pipeline builder {name} failed: {error}");
        }

        self
    }

    pub fn with_layouts(mut self, layouts: &[BL]) -> Self {
        for layout in layouts.iter() {
            self = self.with_layout(layout);
        }

        self
    }

    pub fn with_push_constant_ranges(
        mut self,
        push_constant_ranges: Vec<PushConstantRange>,
    ) -> Self {
        self.push_constant_ranges = push_constant_ranges;

        self
    }

    pub fn build(self) {
        let pipeline_id = self
            .pipeline_cache
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(std::borrow::Cow::Owned(format!("{}--pipeline", self.name))),
                layout: self.layouts,
                push_constant_ranges: self.push_constant_ranges,
                shader: self.shader,
                shader_defs: self.shader_defs,
                entry_point: std::borrow::Cow::Owned(self.entry_point.to_string()),
            });

        self.pipelines_map.insert(self.name, pipeline_id);
    }
}
