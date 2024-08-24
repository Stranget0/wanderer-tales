use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice, Render, RenderApp, RenderSet},
    utils::hashbrown::HashMap,
};

mod buffers_readback;
mod builders;
mod creators;
mod errors;

use buffers_readback::*;
use builders::*;
use bytemuck::AnyBitPattern;
use crossbeam_channel::{Receiver, Sender};
use errors::*;

pub mod prelude {
    pub use super::builders::*;
    pub use super::creators::*;
    pub use super::errors::*;
    pub use super::*;
}

pub type WgslBurritoPluginStr =
    WgslBurritoPlugin<&'static str, &'static str, &'static str, &'static str>;
pub type WgslMainBurritoStr = WgslMainBurrito<&'static str>;
pub type WgslRenderBurritoStr =
    WgslRenderBurrito<&'static str, &'static str, &'static str, &'static str>;

pub struct WgslBurritoPlugin<B, BL, BG, P> {
    _phantom: PhantomData<(B, BL, BG, P)>,
}

impl<B, BL, BG, P> Plugin for WgslBurritoPlugin<B, BL, BG, P>
where
    B: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = B>
        + Send
        + Sync
        + 'static,
    BL: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BL>
        + Send
        + Sync
        + 'static,
    BG: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BG>
        + Send
        + Sync
        + 'static,
    P: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = P>
        + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(WgslMainBurrito::<B>::new());
    }
    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .insert_resource(WgslRenderBurrito::<B, BL, BG, P>::new())
            .add_systems(
                Render,
                // We need to run it after the render graph is done
                // because this needs to happen after submit()
                map_and_read_buffer::<B, BL, BG, P>.after(RenderSet::Render),
            );
    }
}

impl<B, BL, BG, P> WgslBurritoPlugin<B, BL, BG, P>
where
    B: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = B>
        + Send
        + Sync
        + 'static,
    BL: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BL>
        + Send
        + Sync
        + 'static,
    BG: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BG>
        + Send
        + Sync
        + 'static,
    P: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = P>
        + Send
        + Sync
        + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct WgslMainBurrito<B>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = B>,
{
    receivers: HashMap<B, ReadbackReceiver>,
}

#[derive(Resource)]
pub struct WgslRenderBurrito<B, BL, BG, P> {
    buffers: HashMap<B, Buffer>,
    buffers_readback: HashMap<B, ReadbackBuffer>,
    senders: HashMap<B, ReadbackSender>,
    layouts: HashMap<BL, BindGroupLayout>,
    bind_groups: HashMap<BG, BindGroup>,
    pipelines: HashMap<P, CachedComputePipelineId>,
}

pub fn map_and_read_buffer<B, BL, BG, P>(
    device: Res<RenderDevice>,
    wgsl: Res<WgslRenderBurrito<B, BL, BG, P>>,
) where
    B: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = B>
        + Send
        + Sync
        + 'static,
    BL: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BL>
        + Send
        + Sync
        + 'static,
    BG: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = BG>
        + Send
        + Sync
        + 'static,
    P: PartialEq
        + Eq
        + std::hash::Hash
        + std::fmt::Display
        + ToOwned<Owned = P>
        + Send
        + Sync
        + 'static,
{
    for key in wgsl.readback_buffer_keys() {
        match wgsl.send_buffer_to_main(key, device.as_ref()) {
            Ok(_) => {}
            Err(err) => error!("error sending buffer {key}: {err}"),
        }
    }
}

impl<B> WgslMainBurrito<B>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = B>,
{
    pub fn new() -> Self {
        Self {
            receivers: HashMap::default(),
        }
    }

    pub fn insert_receiver(&mut self, buffer_name: B, receiver: Receiver<Vec<u8>>) -> &mut Self {
        self.receivers
            .insert(buffer_name, ReadbackReceiver(receiver));

        self
    }

    pub fn insert_many_receivers(&mut self, receivers: Vec<(B, Receiver<Vec<u8>>)>) -> &mut Self {
        for (buffer_name, receiver) in receivers {
            self.insert_receiver(buffer_name, receiver);
        }

        self
    }

    pub fn try_receive<T: AnyBitPattern>(&self, buffer_name: B) -> Option<T> {
        let result = match self.receivers.get(&buffer_name) {
            Some(receiver) => receiver.try_read::<T>(),
            None => Err(BindGroupBuilderError::no_receiver_found(
                buffer_name.to_owned(),
            )),
        };

        match result {
            Ok(data) => data,
            Err(error) => {
                error!("error reading buffer {buffer_name}: {error}");
                None
            }
        }
    }

    pub fn try_receive_vec<T: AnyBitPattern>(&self, buffer_name: B) -> Option<Vec<T>> {
        let result = match self.receivers.get(&buffer_name) {
            Some(receiver) => receiver.try_read_vec::<T>(),
            None => Err(BindGroupBuilderError::no_receiver_found(
                buffer_name.to_owned(),
            )),
        };

        match result {
            Ok(data) => data,
            Err(error) => {
                error!("error reading buffer {buffer_name}: {error}");
                None
            }
        }
    }
}

impl<B, BL, BG, P> WgslRenderBurrito<B, BL, BG, P>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = B>,
    BL: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = BL>,
    BG: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = BG>,
    P: PartialEq + Eq + std::hash::Hash + std::fmt::Display + ToOwned<Owned = P>,
{
    pub fn new() -> Self {
        Self {
            buffers: HashMap::default(),
            buffers_readback: HashMap::default(),
            senders: HashMap::default(),
            layouts: HashMap::default(),
            bind_groups: HashMap::default(),
            pipelines: HashMap::default(),
        }
    }

    pub fn start_create_buffers<'a, 'b>(
        &'b mut self,
        device: &'a RenderDevice,
    ) -> BufferMutateBuilder<'a, 'b, 'b, B> {
        BufferMutateBuilder::new(device, &mut self.buffers, &mut self.buffers_readback)
    }
    pub fn insert_buffer(&mut self, name: B, buffer: Buffer) -> &mut Self {
        self.buffers.insert(name, buffer);
        self
    }
    pub fn insert_once_buffer(&mut self, name: B, buffer: Buffer) -> &mut Self {
        if self.has_buffer(&name) {
            return self;
        }
        self.insert_buffer(name, buffer);
        self
    }

    pub fn get_buffer(&self, buffer_name: &B) -> Option<&Buffer> {
        self.buffers.get(buffer_name)
    }
    pub fn has_buffer(&self, buffer_name: &B) -> bool {
        self.buffers.contains_key(buffer_name)
    }
    pub fn buffer_keys(&self) -> impl Iterator<Item = &B> {
        self.buffers.keys()
    }
    pub fn get_buffer_readback(&self, buffer_name: &B) -> Option<&ReadbackBuffer> {
        self.buffers_readback.get(buffer_name)
    }
    pub fn has_buffer_readback(&self, buffer_name: &B) -> bool {
        self.buffers_readback.contains_key(buffer_name)
    }
    pub fn readback_buffer_keys(&self) -> impl Iterator<Item = &B> {
        self.buffers_readback.keys()
    }

    pub fn start_create_layout<'a, 'b>(
        &'a mut self,
        name: BL,
        visibility: ShaderStages,
        device: &'b RenderDevice,
    ) -> BindLayoutMutateBuilder<'b, 'a, BL> {
        BindLayoutMutateBuilder::new(device, name, visibility, &mut self.layouts)
    }
    pub fn insert_layout(&mut self, key: BL, layout: BindGroupLayout) {
        self.layouts.insert(key, layout);
    }
    pub fn insert_once_layout(&mut self, key: BL, layout: BindGroupLayout) -> &mut Self {
        if self.has_layout(&key) {
            return self;
        }

        self.insert_layout(key, layout);

        self
    }
    pub fn get_layout(&self, key: &BL) -> Option<&BindGroupLayout> {
        self.layouts.get(key)
    }
    pub fn has_layout(&self, key: &BL) -> bool {
        self.layouts.contains_key(key)
    }
    pub fn layout_keys(&self) -> impl Iterator<Item = &BL> {
        self.layouts.keys()
    }

    pub fn insert_sender(&mut self, buffer_name: B, sender: Sender<Vec<u8>>) -> &mut Self {
        self.senders.insert(buffer_name, ReadbackSender(sender));

        self
    }
    pub fn insert_many_senders(&mut self, senders: Vec<(B, Sender<Vec<u8>>)>) -> &mut Self {
        for (buffer_name, sender) in senders {
            self.insert_sender(buffer_name, sender);
        }

        self
    }

    pub fn get_sender(&self, buffer_name: &B) -> Option<&ReadbackSender> {
        self.senders.get(buffer_name)
    }
    pub fn has_sender(&self, buffer_name: &B) -> bool {
        self.senders.contains_key(buffer_name)
    }
    pub fn sender_keys(&self) -> impl Iterator<Item = &B> {
        self.senders.keys()
    }

    pub fn create_bind_group(
        &mut self,
        bind_group_name: BG,
        device: &RenderDevice,
        layout_name: BL,
        buffer_entries: &[B],
    ) -> &mut Self {
        let query_result = match self.layouts.get(&layout_name) {
            Some(layout) => bind_group(
                bind_group_name.to_owned(),
                device,
                layout,
                &self.buffers,
                buffer_entries,
            ),
            None => Err(BindGroupBuilderError::no_layout_found(layout_name)),
        };

        match query_result {
            Ok(bind_group) => {
                self.bind_groups.insert(bind_group_name, bind_group);
            }
            Err(error) => error!("binding group failed: {error}"),
        };

        self
    }

    pub fn create_once_bind_group(
        &mut self,
        bind_group_name: BG,
        device: &RenderDevice,
        layout_name: BL,
        buffer_entries: &[B],
    ) -> &mut Self {
        if self.has_bind_group(&bind_group_name) {
            return self;
        }

        self.create_bind_group(bind_group_name, device, layout_name, buffer_entries);

        self
    }
    pub fn get_bind_group(&self, bind_group_name: &BG) -> Option<&BindGroup> {
        self.bind_groups.get(bind_group_name)
    }
    pub fn has_bind_group(&self, bind_group_name: &BG) -> bool {
        self.bind_groups.contains_key(bind_group_name)
    }
    pub fn bind_group_keys(&self) -> impl Iterator<Item = &BG> {
        self.bind_groups.keys()
    }

    pub fn start_create_pipeline<'a, 'b, 'c>(
        &'a mut self,
        name: P,
        pipeline_cache: &'b PipelineCache,
        shader: Handle<Shader>,
        entry_point: &'c str,
    ) -> PipelineMutateBuilder<'b, 'a, 'c, 'a, P, BL> {
        PipelineMutateBuilder::new(
            name,
            pipeline_cache,
            shader,
            &self.layouts,
            entry_point,
            &mut self.pipelines,
        )
    }
    pub fn insert_pipeline(&mut self, name: P, pipeline: CachedComputePipelineId) -> &mut Self {
        self.pipelines.insert(name, pipeline);
        self
    }
    pub fn insert_once_pipeline(
        &mut self,
        name: P,
        pipeline: CachedComputePipelineId,
    ) -> &mut Self {
        if self.has_pipeline(&name) {
            return self;
        }
        self.insert_pipeline(name, pipeline);
        self
    }

    pub fn get_pipeline(&self, pipeline_name: &P) -> Option<&CachedComputePipelineId> {
        self.pipelines.get(pipeline_name)
    }
    pub fn has_pipeline(&self, pipeline_name: &P) -> bool {
        self.pipelines.contains_key(pipeline_name)
    }
    pub fn pipeline_keys(&self) -> impl Iterator<Item = &P> {
        self.pipelines.keys()
    }

    pub fn copy_to_readback_buffer(&self, buffer_name: &B, encoder: &mut CommandEncoder) {
        let Some(gpu_buffer) = self.buffers.get(buffer_name) else {
            let error = BindGroupBuilderError::no_buffer_found(buffer_name);
            error!("error filling readback buffer {buffer_name}: {error}");
            return;
        };
        let Some(cpu_buffer) = self.buffers_readback.get(buffer_name) else {
            let error = BindGroupBuilderError::no_readback_buffer_found(buffer_name);
            error!("error filling readback buffer {buffer_name}: {error}");
            return;
        };
        if gpu_buffer.size() != cpu_buffer.buffer.size() {
            warn!("buffer {buffer_name} size mismatch");
        }

        encoder.copy_buffer_to_buffer(gpu_buffer, 0, &cpu_buffer.buffer, 0, gpu_buffer.size());
    }

    pub fn send_buffer_to_main(
        &self,
        buffer_key: &B,
        device: &RenderDevice,
    ) -> Result<(), BindGroupBuilderError> {
        let Some(sender) = self.senders.get(buffer_key) else {
            return Err(BindGroupBuilderError::no_sender_found(buffer_key));
        };
        let Some(buffer) = self.buffers_readback.get(buffer_key) else {
            return Err(BindGroupBuilderError::no_readback_buffer_found(buffer_key));
        };

        sender.try_send(device, buffer)
    }
}
