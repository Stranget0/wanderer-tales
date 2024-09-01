use bevy::render::render_resource::*;
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, render::renderer::RenderDevice};
use bytemuck::AnyBitPattern;

use super::prelude::*;

#[derive(Resource)]
pub struct WgslMainBurrito<B>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = B>,
{
    pub(crate) receivers: HashMap<B, ReadbackReceiver>,
}

#[derive(Resource)]
pub struct WgslRenderBurrito<B, BL, BG, P> {
    pub(crate) buffers: HashMap<B, Buffer>,
    pub(crate) buffers_readback: HashMap<B, ReadbackBuffer>,
    pub(crate) senders: HashMap<B, ReadbackSender>,
    pub(crate) layouts: HashMap<BL, BindGroupLayout>,
    pub(crate) bind_groups: HashMap<BG, BindGroup>,
    pub(crate) pipelines: HashMap<P, CachedComputePipelineId>,
}

pub(crate) fn map_and_read_buffer<B, BL, BG, P>(
    device: Res<RenderDevice>,
    mut wgsl: ResMut<WgslRenderBurrito<B, BL, BG, P>>,
) where
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
    let mut sent_buffers = Vec::new();
    for name in wgsl.readback_buffer_keys() {
        let buffer = wgsl
            .buffers_readback
            .get(name)
            .ok_or_else(|| (BindGroupBuilderError::no_readback_buffer_found(name)));

        let sender = wgsl
            .senders
            .get(name)
            .ok_or_else(|| (BindGroupBuilderError::no_sender_found(name)));

        let buffer_sender = match (buffer, sender) {
            (Ok(buffer), Ok(sender)) => Ok((buffer, sender)),
            (Err(err), _) | (_, Err(err)) => Err(err),
        };

        let sent_result = buffer_sender.and_then(|(buffer, sender)| {
            if buffer.has_changed() {
                sender.try_send(device.as_ref(), buffer).map(|_| true)
            } else {
                Ok(false)
            }
        });

        match sent_result {
            Ok(true) => {
                info!("buffer {name:?} sent to main");
                sent_buffers.push(name.to_owned());
            }
            Err(err) => error!("error sending buffer {name:?}: {err}"),
            _ => {}
        }
    }

    for name in sent_buffers {
        info!("marking {name:?} as sent");
        wgsl.get_buffer_readback_mut(&name).unwrap().mark_sent();
    }
}

impl<B: std::fmt::Debug + std::hash::Hash + PartialEq + Eq + ToOwned<Owned = B>> Default
    for WgslMainBurrito<B>
{
    fn default() -> Self {
        Self {
            receivers: HashMap::new(),
        }
    }
}

impl<B> WgslMainBurrito<B>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = B>,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_receiver(
        &mut self,
        buffer_name: B,
        receiver: crossbeam_channel::Receiver<Vec<u8>>,
    ) -> &mut Self {
        self.receivers
            .insert(buffer_name, ReadbackReceiver(receiver));

        self
    }

    pub fn insert_many_receivers(
        &mut self,
        receivers: Vec<(B, crossbeam_channel::Receiver<Vec<u8>>)>,
    ) -> &mut Self {
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
                error!("error reading buffer {buffer_name:?}: {error}");
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
                error!("error reading buffer {buffer_name:?}: {error}");
                None
            }
        }
    }
}

impl<B, BL, BG, P> Default for WgslRenderBurrito<B, BL, BG, P>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = B>,
    BL: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = BL>,
    BG: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = BG>,
    P: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = P>,
{
    fn default() -> Self {
        Self {
            buffers: HashMap::new(),
            buffers_readback: HashMap::new(),
            senders: HashMap::new(),
            layouts: HashMap::new(),
            bind_groups: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }
}

impl<B, BL, BG, P> WgslRenderBurrito<B, BL, BG, P>
where
    B: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = B>,
    BL: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = BL>,
    BG: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = BG>,
    P: PartialEq + Eq + std::hash::Hash + std::fmt::Debug + ToOwned<Owned = P>,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_create_buffers<'a, 'b>(
        &'b mut self,
        device: &'a RenderDevice,
    ) -> BufferMutateBuilder<'a, 'b, 'b, B> {
        BufferMutateBuilder::new(device, &mut self.buffers, &mut self.buffers_readback)
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
    pub fn get_buffer_readback_mut(&mut self, buffer_name: &B) -> Option<&mut ReadbackBuffer> {
        self.buffers_readback.get_mut(buffer_name)
    }
    pub fn has_buffer_readback(&self, buffer_name: &B) -> bool {
        self.buffers_readback.contains_key(buffer_name)
    }
    pub fn readback_buffer_keys(&self) -> impl Iterator<Item = &B> {
        self.buffers_readback.keys()
    }

    pub fn builder_layout<'a>(
        &self,
        name: BL,
        device: &'a RenderDevice,
        visibility: ShaderStages,
    ) -> BindLayoutBuilder<'a, BL> {
        BindLayoutBuilder::new(device, name, visibility)
    }
    pub fn insert_layout(&mut self, key: BL, layout: BindGroupLayout) -> &mut Self {
        self.layouts.insert(key, layout);
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

    pub fn insert_bind_group(&mut self, bind_group_name: BG, bind_group: BindGroup) -> &mut Self {
        self.bind_groups.insert(bind_group_name, bind_group);
        self
    }

    pub(crate) fn get_buffers(
        &self,
        buffer_entries: &[B],
    ) -> Result<Vec<&Buffer>, BindGroupBuilderError> {
        let mut buffers = Vec::with_capacity(buffer_entries.len());
        for name in buffer_entries.iter() {
            let Some(buffer) = self.buffers.get(name) else {
                return Err(BindGroupBuilderError::no_buffer_found(name));
            };
            buffers.push(buffer);
        }

        Ok(buffers)
    }

    pub fn create_bind_group(
        &mut self,
        bind_group_name: BG,
        device: &RenderDevice,
        layout_name: BL,
        buffer_names: &[B],
    ) -> &mut Self {
        let query_result = match (
            self.layouts.get(&layout_name),
            self.get_buffers(buffer_names)
                .and_then(|buffers| buffers_to_bind_entries(buffers, buffer_names)),
        ) {
            (Some(layout), Ok(entries)) => Ok(creators::create_bind_group(
                device,
                bind_group_name.to_owned(),
                layout,
                entries,
            )),
            (None, _) => Err(BindGroupBuilderError::no_layout_found(layout_name)),
            (_, Err(error)) => Err(error),
        };

        match query_result {
            Ok(bind_group) => {
                self.bind_groups.insert(bind_group_name, bind_group);
            }
            Err(error) => error!("binding group failed: {error}"),
        };

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

    pub fn builder_pipeline<'a, 'b>(
        &'a self,
        name: P,
        shader: Handle<Shader>,
        entry_point: &'b str,
    ) -> PipelineBuilder<'b, 'a, P, BL> {
        PipelineBuilder::new(name, shader, entry_point, &self.layouts)
    }
    pub fn insert_pipeline(&mut self, name: P, pipeline: CachedComputePipelineId) -> &mut Self {
        self.pipelines.insert(name, pipeline);
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

    pub fn copy_to_readback_buffer(&self, encoder: &mut CommandEncoder, buffer_name: &B) {
        let Some(gpu_buffer) = self.buffers.get(buffer_name) else {
            let error = BindGroupBuilderError::no_buffer_found(buffer_name);
            error!("error filling readback buffer {buffer_name:?}: {error}");
            return;
        };
        let Some(cpu_buffer) = self.buffers_readback.get(buffer_name) else {
            let error = BindGroupBuilderError::no_readback_buffer_found(buffer_name);
            error!("error filling readback buffer {buffer_name:?}: {error}");
            return;
        };
        if gpu_buffer.size() != cpu_buffer.buffer.size() {
            warn!("buffer {buffer_name:?} size mismatch");
        }

        encoder.copy_buffer_to_buffer(gpu_buffer, 0, &cpu_buffer.buffer, 0, gpu_buffer.size());
    }

    pub fn copy_buffer_to_buffer(&self, encoder: &mut CommandEncoder, from: &B, to: &B) {
        let buffers = match (self.buffers.get(from), self.buffers.get(to)) {
            (Some(from), Some(to)) => {
                if from.size() != to.size() {
                    Err(BindGroupBuilderError::BufferSizeMismatch(
                        from.size(),
                        to.size(),
                    ))
                } else {
                    Ok((from, to))
                }
            }
            (None, _) => Err(BindGroupBuilderError::no_buffer_found(from)),
            _ => Err(BindGroupBuilderError::no_buffer_found(to)),
        };

        match buffers {
            Ok((from, to)) => {
                encoder.copy_buffer_to_buffer(from, 0, to, 0, to.size());
            }
            Err(error) => {
                error!("error copying buffer {from:?} to {to:?}: {error}");
            }
        }
    }

    pub fn insert_sender(
        &mut self,
        buffer_name: B,
        sender: crossbeam_channel::Sender<Vec<u8>>,
    ) -> &mut Self {
        self.senders.insert(buffer_name, ReadbackSender(sender));

        self
    }
    pub fn insert_many_senders(
        &mut self,
        senders: Vec<(B, crossbeam_channel::Sender<Vec<u8>>)>,
    ) -> &mut Self {
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
}

fn buffers_to_bind_entries<'a, B: std::fmt::Debug + ToOwned<Owned = B>>(
    buffers: Vec<&'a Buffer>,
    buffer_names: &[B],
) -> Result<Vec<BindGroupEntry<'a>>, BindGroupBuilderError> {
    let mut entries = Vec::with_capacity(buffers.len());
    for (index, buffer) in buffers.iter().enumerate() {
        if buffer.size() == 0 {
            return Err(BindGroupBuilderError::empty_buffer(
                buffer_names[index].to_owned(),
            ));
        }
        entries.push(BindGroupEntry {
            binding: index as u32,
            resource: buffer.as_entire_binding(),
        });
    }
    Ok(entries)
}
