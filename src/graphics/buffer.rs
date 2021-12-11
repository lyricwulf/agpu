mod builder;
pub use builder::*;

mod binding;
pub use binding::*;

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
    pub(crate) label: String,
    pub(crate) inner: wgpu::Buffer,
    pub(crate) usages: wgpu::BufferUsages,
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
    pub fn download_immediately(
        &self,
    ) -> Result<wgpu::util::DownloadBuffer, wgpu::BufferAsyncError> {
        let fut = wgpu::util::DownloadBuffer::read_buffer(
            &self.gpu.device,
            &self.gpu.queue,
            &self.inner.slice(..),
        );
        self.gpu.poll(wgpu::Maintain::Wait);
        block_on(fut)
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    // Writes the data to the buffer
    // Resizes the buffer if the data is larger than the current buffer size
    pub fn write<T>(&mut self, data: &[T])
    where
        T: bytemuck::Pod,
    {
        let data = bytemuck::cast_slice(data);
        // If the buffer is too small, it will be resized
        if self.size < data.len() as _ {
            self.resize_impl(data.len() as _, false);
        }
        self.gpu.queue.write_buffer(&self.inner, 0, data);
    }

    // Writes the data to the buffer
    // Will panic if the data is larger than the current buffer size
    pub fn write_unchecked<T>(&self, data: &[T])
    where
        T: bytemuck::Pod,
    {
        let data = bytemuck::cast_slice(data);
        self.gpu.queue.write_buffer(&self.inner, 0, data);
    }

    fn resize_impl(&mut self, size: u64, copy_contents: bool) {
        if copy_contents {
            self.usages |= wgpu::BufferUsages::COPY_DST
        };
        // Create the new buffer
        let new_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size,
            usage: self.usages,
            mapped_at_creation: false,
        });
        if copy_contents {
            // Copy the contents of the old buffer to the new buffer
            let mut encoder = self
                .gpu
                .create_command_encoder("agpu::Buffer::resize_impl() command encoder");
            self.copy_to(&mut encoder, &new_buffer);
            self.gpu.queue.submit([encoder.finish()]);
        };
        // Destroy the old buffer
        self.inner.destroy();
        // Update the inner buffer
        self.inner = new_buffer;
        // Update the size
        self.size = size;
    }

    pub fn resize(&mut self, size: u64) {
        self.resize_impl(size, true);
    }

    pub fn copy_to(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::Buffer) {
        encoder.copy_buffer_to_buffer(&self.inner, 0, target, 0, self.size);
    }
}
