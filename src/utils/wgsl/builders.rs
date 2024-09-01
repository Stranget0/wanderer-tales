use super::buffers_readback::*;
use super::creators;
use super::errors::BindGroupBuilderError;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::{
    render::{
        render_resource::{
            binding_types, encase::internal::WriteInto, BindGroupLayoutEntry,
            BindGroupLayoutEntryBuilder, Buffer, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
    utils::hashbrown::HashMap,
};

pub struct BufferMutateBuilder<
    'a,
    'b,
    'c,
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned,
> {
    device: &'a RenderDevice,
    buffer_map: &'b mut HashMap<B, Buffer>,
    readback_buffer_map: &'c mut HashMap<B, ReadbackBuffer>,
}

impl<'a, 'b, 'c, B> BufferMutateBuilder<'a, 'b, 'c, B>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = B>,
{
    pub fn new(
        device: &'a RenderDevice,
        buffer_map: &'b mut HashMap<B, Buffer>,
        readback_buffer_map: &'c mut HashMap<B, ReadbackBuffer>,
    ) -> Self {
        Self {
            device,
            buffer_map,
            readback_buffer_map,
        }
    }

    pub fn create_once_empty_storage_readable(self, name: B, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage_readable(name, size)
    }
    pub fn create_once_storage_readable<T: WriteInto + ShaderType>(
        self,
        name: B,
        payload: &T,
    ) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_storage_readable(name, payload)
    }
    pub fn create_once_empty_uniform<T: WriteInto + ShaderType>(self, name: B, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_uniform::<T>(name, size)
    }
    pub fn create_once_empty_storage(self, name: B, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage(name, size)
    }
    pub fn create_once_empty_storage_rw(self, name: B, size: u64) -> Self {
        if self.has_buffer(&name) {
            return self;
        }

        self.create_empty_storage_rw(name, size)
    }

    pub fn create_uniform<T: WriteInto + ShaderType>(mut self, name: B, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        T::assert_uniform_compat();
        let device = self.get_device();
        let buffer = creators::uniform_buffer(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    pub fn create_storage<T: WriteInto + ShaderType>(mut self, name: B, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    pub fn create_storage_rw<T: WriteInto + ShaderType>(mut self, name: B, payload: &T) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer_rw(device, &name, payload);

        self.insert_buffer(name, buffer);

        self
    }

    pub fn create_storage_readable<T: WriteInto + ShaderType>(
        mut self,
        name: B,
        payload: &T,
    ) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_buffer_rw(device, &name, payload);
        let size = buffer.size();
        let readback_buffer = creators::cpu_buffer::<B>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    pub fn create_empty_storage_readable(mut self, name: B, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty_rw(device, &name, size);
        let size = buffer.size();
        let readback_buffer = creators::cpu_buffer::<B>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    pub fn create_empty_uniform<T: WriteInto + ShaderType>(mut self, name: B, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::uniform_empty::<B, T>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    pub fn create_empty_storage(mut self, name: B, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty::<B>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    pub fn create_empty_storage_rw(mut self, name: B, size: u64) -> Self
    where
        Self: std::marker::Sized,
    {
        let device = self.get_device();
        let buffer = creators::storage_empty_rw::<B>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    fn has_buffer(&self, name: &B) -> bool {
        self.buffer_map.contains_key(name)
    }

    fn insert_buffer(&mut self, name: B, buffer: Buffer) {
        self.buffer_map.insert(name, buffer);
    }

    fn insert_readback_buffer(&mut self, name: B, buffer: Buffer) {
        self.readback_buffer_map
            .insert(name, ReadbackBuffer::new(buffer));
    }

    fn get_device(&self) -> &RenderDevice {
        self.device
    }
}

pub struct BindLayoutBuilder<
    'a,
    N: std::fmt::Debug + PartialEq + Eq + std::hash::Hash + ToOwned<Owned = N>,
> {
    name: N,
    device: &'a RenderDevice,
    entries: Vec<BindGroupLayoutEntry>,
    visibility: ShaderStages,
}

impl<'a, N> BindLayoutBuilder<'a, N>
where
    N: std::fmt::Debug + PartialEq + Eq + std::hash::Hash + ToOwned<Owned = N>,
{
    pub fn new(device: &'a RenderDevice, name: N, visibility: ShaderStages) -> Self {
        Self {
            device,
            name,
            visibility,
            entries: Vec::new(),
        }
    }

    fn push_binding(&mut self, binding_builder: BindGroupLayoutEntryBuilder) {
        let binding_index = self.entries.len() as u32;
        let name = &self.name;
        let binding = binding_builder.build(binding_index, self.visibility);
        info!("layout {name:?} binding {binding_index} {binding:?}");

        self.entries.push(binding);
    }

    pub fn with_uniform_slot<T: WriteInto + ShaderType>(mut self) -> Self {
        T::assert_uniform_compat();

        self.push_binding(binding_types::uniform_buffer::<T>(false));

        self
    }

    pub fn with_storage_slot<T: WriteInto + ShaderType>(mut self) -> Self {
        self.push_binding(binding_types::storage_buffer::<T>(false));

        self
    }

    pub fn build(self) -> BindGroupLayout {
        let device = &self.device;
        let entries = self.entries.as_slice();
        let name = self.name;

        creators::create_bind_group_layout(device, name.to_owned(), entries)
    }
}

pub struct PipelineBuilder<
    'a,
    'c,
    P: std::fmt::Debug,
    BL: PartialEq + Eq + std::hash::Hash + std::fmt::Debug,
> {
    entry_point: &'a str,
    layouts_map: &'c HashMap<BL, BindGroupLayout>,
    shader: Handle<Shader>,
    shader_defs: Vec<ShaderDefVal>,
    push_constant_ranges: Vec<PushConstantRange>,
    layouts: Vec<BindGroupLayout>,
    name: P,
}

impl<'a, 'c, P: std::fmt::Debug, BL: PartialEq + Eq + std::hash::Hash + std::fmt::Debug>
    PipelineBuilder<'a, 'c, P, BL>
{
    pub fn new(
        name: P,
        shader: Handle<Shader>,
        entry_point: &'a str,
        layouts_map: &'c HashMap<BL, BindGroupLayout>,
    ) -> Self {
        Self {
            shader,
            layouts_map,
            name,
            entry_point,
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
            error!("pipeline builder {name:?} failed: {error}");
        }

        self
    }

    pub fn with_layout_value(mut self, layout: BindGroupLayout) -> Self {
        self.layouts.push(layout);
        self
    }

    pub fn with_push_constant_ranges(
        mut self,
        push_constant_ranges: Vec<PushConstantRange>,
    ) -> Self {
        self.push_constant_ranges = push_constant_ranges;

        self
    }

    pub fn build(self) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some(std::borrow::Cow::Owned(format!(
                "{:?}--pipeline",
                self.name
            ))),
            layout: self.layouts,
            push_constant_ranges: self.push_constant_ranges,
            shader: self.shader,
            shader_defs: self.shader_defs,
            entry_point: std::borrow::Cow::Owned(self.entry_point.to_string()),
        }
    }
}
