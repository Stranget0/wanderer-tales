use bevy::prelude::*;
use bevy::render::{
    render_resource::{Buffer, Maintain, MapMode},
    renderer::RenderDevice,
};
use crossbeam_channel::Sender;

pub(crate) struct ReadbackBuffer {
    pub buffer: Buffer,
}

impl ReadbackBuffer {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
    pub fn poll_map_and_read<T: FromChunks>(&self, device: &RenderDevice, sender: &Sender<T>) {
        poll_map_and_read(device, &self.buffer, sender);
    }
}

pub(crate) fn poll_map_and_read<T: FromChunks>(
    device: &RenderDevice,
    buffer: &Buffer,
    sender: &Sender<T>,
) {
    {
        let slice = buffer.slice(..);

        let (s, r) = crossbeam_channel::unbounded::<()>();

        // This isnt async and we cannot await it, this is why we use a channel
        slice.map_async(MapMode::Read, move |r| match r {
            // This will execute once the gpu is ready, so after the call to poll()
            Ok(_) => s.send(()).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        // This blocks until the gpu is done executing everything
        device.poll(Maintain::wait()).panic_on_timeout();
        // This blocks until the buffer is mapped
        r.recv().expect("Failed to receive the map_async message");
        let view = slice.get_mapped_range();
        let chunks = view.chunks(std::mem::size_of::<T>());
        // TODO: handle error
        match sender.send(T::from_chunks(chunks)) {
            Ok(_) => (),
            Err(err) => error!("Failed to send data to main world {err}"),
        }
    }

    buffer.unmap();
}

impl FromChunks for Vec<u32> {
    fn from_chunks(chunks: std::slice::Chunks<u8>) -> Self {
        chunks
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("should be a u32")))
            .collect::<Vec<u32>>()
    }
}

impl FromChunks for Vec<f32> {
    fn from_chunks(chunks: std::slice::Chunks<u8>) -> Self {
        chunks
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("should be a f32")))
            .collect::<Vec<f32>>()
    }
}

impl FromChunks for Vec<i32> {
    fn from_chunks(chunks: std::slice::Chunks<u8>) -> Self {
        chunks
            .map(|chunk| i32::from_ne_bytes(chunk.try_into().expect("should be a i32")))
            .collect::<Vec<i32>>()
    }
}

pub trait FromChunks {
    fn from_chunks(chunks: std::slice::Chunks<u8>) -> Self;
}
