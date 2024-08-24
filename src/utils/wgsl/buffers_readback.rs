use bevy::prelude::*;
use bevy::render::{
    render_resource::{Buffer, Maintain, MapMode},
    renderer::RenderDevice,
};
use crossbeam_channel::*;

use super::BindGroupBuilderError;

#[derive(Resource)]
pub struct ReadbackReceiver(pub Receiver<Vec<u8>>);

#[derive(Resource)]
pub struct ReadbackSender(pub Sender<Vec<u8>>);

pub(crate) struct ReadbackBuffer {
    pub buffer: Buffer,
}

pub trait Readable {
    fn try_read_raw(&self) -> Result<Option<Vec<u8>>, BindGroupBuilderError>;

    fn try_read<T: bytemuck::AnyBitPattern>(&self) -> Result<Option<T>, BindGroupBuilderError> {
        let result = self.try_read_raw()?;
        let Some(bytes) = result else {
            return Ok(None);
        };

        match bytemuck::try_from_bytes::<T>(bytes.as_slice()) {
            Ok(data) => Ok(Some(*data)),
            Err(err) => Err(BindGroupBuilderError::CastFailed(err)),
        }
    }

    fn try_read_vec<T: bytemuck::AnyBitPattern>(
        &self,
    ) -> Result<Option<Vec<T>>, BindGroupBuilderError> {
        let result = self.try_read_raw()?;
        let Some(bytes) = result else {
            return Ok(None);
        };

        match bytemuck::try_cast_slice::<u8, T>(&bytes) {
            Ok(data) => Ok(Some(data.to_vec())),
            Err(err) => Err(BindGroupBuilderError::CastFailed(err)),
        }
    }
}

impl ReadbackReceiver {
    pub fn new(receiver: Receiver<Vec<u8>>) -> Self {
        Self(receiver)
    }
}

impl Readable for ReadbackReceiver {
    fn try_read_raw(&self) -> Result<Option<Vec<u8>>, BindGroupBuilderError> {
        let data = self.0.try_recv();
        match data {
            Ok(data) => Ok(Some(data)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(BindGroupBuilderError::ReceiverDistonnected),
        }
    }
}

impl ReadbackBuffer {
    #[inline]
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    #[inline]
    pub fn unmap(&self) {
        self.buffer.unmap();
    }
}

impl ReadbackSender {
    #[inline]
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self(sender)
    }

    pub fn try_send(
        &self,
        device: &RenderDevice,
        buffer: &ReadbackBuffer,
    ) -> Result<(), BindGroupBuilderError> {
        let data = poll_map_and_read(device, &buffer.buffer);
        let result = self
            .0
            .send(data)
            .or_else(|err| Err(BindGroupBuilderError::SendFailed(err)));
        // We need to make sure all `BufferView`'s are dropped before we do what we're about
        // to do.
        // Unmap so that we can copy to the staging buffer in the next iteration.
        buffer.unmap();

        result
    }
}

pub(crate) fn poll_map_and_read(device: &RenderDevice, buffer: &Buffer) -> Vec<u8> {
    // Finally time to get our data back from the gpu.
    // First we get a buffer slice which represents a chunk of the buffer (which we
    // can't access yet).
    // We want the whole thing so use unbounded range.
    let buffer_slice = buffer.slice(..);

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
    device.poll(Maintain::wait()).panic_on_timeout();

    // This blocks until the buffer is mapped
    r.recv().expect("Failed to receive the map_async message");

    buffer_slice.get_mapped_range().to_vec()
}
