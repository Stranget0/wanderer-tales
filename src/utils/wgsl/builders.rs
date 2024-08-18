use super::buffers;
use super::buffers_readback::*;
use super::errors::BindGroupBuilderError;
use bevy::prelude::Resource;
use bevy::render::render_resource::BindGroupLayout;
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

pub struct BufferBuilder<'a, K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned> {
    device: &'a RenderDevice,
    buffer_map: HashMap<K, Buffer>,
    readback_buffer_map: HashMap<K, ReadbackBuffer>,
}

impl<'a, K> BufferBuilder<'a, K>
where
    K: PartialEq + Eq + std::hash::Hash + std::fmt::Display + std::fmt::Debug + ToOwned<Owned = K>,
{
    pub fn new(device: &'a RenderDevice) -> Self {
        Self {
            device,
            buffer_map: HashMap::default(),
            readback_buffer_map: HashMap::default(),
        }
    }
    fn insert_buffer(&mut self, name: K, buffer: Buffer) {
        self.buffer_map.insert(name, buffer);
    }

    fn insert_readback_buffer(&mut self, name: K, buffer: Buffer) {
        self.readback_buffer_map
            .insert(name, ReadbackBuffer::new(buffer));
    }

    pub fn create_uniform<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self {
        T::assert_uniform_compat();
        let device = self.device;
        let buffer = buffers::uniform_buffer(device, &name, payload);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    pub fn create_storage<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self {
        let device = self.device;
        let buffer = buffers::storage_buffer(device, &name, payload);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }
    pub fn create_storage_rw<T: WriteInto + ShaderType>(mut self, name: K, payload: &T) -> Self {
        let device = self.device;
        let buffer = buffers::storage_buffer_rw(device, &name, payload);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    pub fn create_storage_readable<T: WriteInto + ShaderType + FromChunks>(
        mut self,
        name: K,
        payload: &T,
    ) -> Self {
        let device = self.device;
        let buffer = buffers::storage_buffer_rw(device, &name, payload);
        let size = buffer.size();
        let readback_buffer = buffers::cpu_buffer::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    pub fn create_empty_storage_readable(mut self, name: K, size: u64) -> Self {
        let device = self.device;
        let buffer = buffers::storage_empty_rw(device, &name, size);
        let size = buffer.size();
        let readback_buffer = buffers::cpu_buffer::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);
        self.insert_readback_buffer(name, readback_buffer);

        self
    }

    pub fn create_empty_uniform<T: WriteInto + ShaderType>(mut self, name: K, size: u64) -> Self {
        let device = self.device;
        let buffer = buffers::uniform_empty::<K, T>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }
    pub fn create_empty_storage<T: WriteInto + ShaderType>(mut self, name: K, size: u64) -> Self {
        let device = self.device;
        let buffer = buffers::storage_empty::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }
    pub fn create_empty_storage_rw<T: WriteInto + ShaderType>(
        mut self,
        name: K,
        size: u64,
    ) -> Self {
        let device = self.device;
        let buffer = buffers::storage_empty_rw::<K>(device, &name, size);

        self.insert_buffer(name.to_owned(), buffer);

        self
    }

    pub fn build(self) -> (HashMap<K, Buffer>, HashMap<K, ReadbackBuffer>) {
        (self.buffer_map, self.readback_buffer_map)
    }
}

pub struct BindLayoutBuilder<'a, N> {
    name: N,
    device: &'a RenderDevice,
    entries: Vec<BindGroupLayoutEntry>,
    visibility: ShaderStages,
}

impl<'a, N> BindLayoutBuilder<'a, N>
where
    N: std::fmt::Display,
{
    pub fn new(device: &'a RenderDevice, name: N, visibility: ShaderStages) -> Self {
        Self {
            device,
            name,
            visibility,
            entries: Vec::new(),
        }
    }

    fn insert_binding(&mut self, binding: BindGroupLayoutEntryBuilder) {
        let binding_index = self.entries.len() as u32;

        self.entries
            .push(binding.build(binding_index, self.visibility));
    }

    pub fn create_uniform<T: WriteInto + ShaderType>(mut self) -> Self {
        T::assert_uniform_compat();

        self.insert_binding(binding_types::uniform_buffer::<T>(false));

        self
    }

    pub fn create_storage<T: WriteInto + ShaderType>(mut self) -> Self {
        self.insert_binding(binding_types::storage_buffer::<T>(false));

        self
    }

    pub fn build(self) -> BindGroupLayout {
        self.device.create_bind_group_layout(
            Some(format!("{}--layout", self.name).as_str()),
            self.entries.as_slice(),
        )
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
) -> Result<BindGroup, BindGroupBuilderError<K>> {
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

    Ok(device.create_bind_group(
        Some(format!("{}--bind-group", bind_group_name).as_str()),
        layout,
        bind_group_entries.as_slice(),
    ))
}

#[derive(Resource)]
pub struct BindGroupBurrito<K: PartialEq + Eq + std::hash::Hash + std::fmt::Display> {
    bind_group: BindGroup,
    layout: BindGroupLayout,
    buffer_map: HashMap<K, Buffer>,
    readback_map: HashMap<K, ReadbackBuffer>,
}
