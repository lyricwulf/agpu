mod builder;
pub use builder::*;

mod binding;

mod view;
use futures::executor::block_on;
pub use view::*;

mod vertex_layout;
pub use vertex_layout::*;

use crate::GpuHandle;
use std::ops::Deref;

/// * Probably best used as `RefCell<Buffer>`
pub struct Buffer {
    pub(crate) gpu: GpuHandle,
    pub(crate) inner: wgpu::Buffer,
    pub size: u64,
}
/// Allows you to use this as a reference to the inner `wgpu::Buffer`
impl Deref for Buffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl Buffer {
    /// # Errors
    /// Errors according to [`wgpu::BufferAsyncError`]
    pub fn download(&self) -> Result<wgpu::util::DownloadBuffer, wgpu::BufferAsyncError> {
        let fut = wgpu::util::DownloadBuffer::read_buffer(
            &self.gpu.device,
            &self.gpu.queue,
            &self.inner.slice(..),
        );
        block_on(fut)
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn write<T>(&self, data: &[T])
    where
        T: bytemuck::Pod,
    {
        self.gpu
            .queue
            .write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
    }
}
