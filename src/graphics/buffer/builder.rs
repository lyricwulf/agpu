use bytemuck::Pod;
use wgpu::util::DeviceExt;

use crate::{Buffer, GpuHandle};

pub enum BufferInitContent<'a> {
    /// The buffer will be initialized with the given data
    Data(&'a [u8]),
    /// The buffer will be initialized with the given size
    Size(u64),
}

pub struct BufferBuilder<'a> {
    pub gpu: GpuHandle,
    pub label: Option<&'a str>,
    pub usage: wgpu::BufferUsages,
}
impl<'a> BufferBuilder<'a> {
    #[must_use]
    pub const fn new(gpu: GpuHandle, label: &'a str) -> Self {
        BufferBuilder {
            gpu,
            label: Some(label),
            usage: wgpu::BufferUsages::empty(),
        }
    }

    /// Set a label that GPU debuggers can display
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Allow a buffer to be the index buffer in a draw operation.
    pub fn as_index_buffer(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::INDEX;
        self
    }

    /// Allow a buffer to be the vertex buffer in a draw operation.
    pub fn as_vertex_buffer(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::VERTEX;
        self
    }

    /// Allow a buffer to be a `BufferBindingType::Uniform` inside a bind group.
    pub fn as_uniform_buffer(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::UNIFORM;
        self
    }

    /// Allow a buffer to be a `BufferBindingType::Storage` inside a bind group.
    pub fn as_storage_buffer(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::STORAGE;
        self
    }

    /// Allow a buffer to be the indirect buffer in an indirect draw call.
    pub fn as_indirect_buffer(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::INDIRECT;
        self
    }

    /// See [`wgpu::BufferUsages::MAP_READ`]
    pub fn allow_map_read(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::MAP_READ;
        self
    }

    /// See [`wgpu::BufferUsages::MAP_WRITE`]
    pub fn allow_map_write(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::MAP_WRITE;
        self
    }

    /// See [`wgpu::BufferUsages::COPY_DST`]
    pub fn allow_copy_to(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_DST;
        self
    }

    /// See [`wgpu::BufferUsages::COPY_SRC`]
    pub fn allow_copy_from(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_SRC;
        self
    }

    /// Sets the usage of the buffer
    /// See also `add_usage` and `rm_usage`
    pub fn with_usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.usage = usage;
        self
    }
    /// Adds the usage flag to the buffer
    /// See also `with_usage` and `add_usage`
    pub fn add_usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.usage |= usage;
        self
    }
    /// Removes the usage flag from the buffer
    /// See also `with_usage` and `add_usage`
    pub fn rm_usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.usage &= !usage;
        self
    }

    // This is used by build() and build_and_map() for our convenience
    fn create_impl(&self, init: BufferInitContent) -> (wgpu::Buffer, u64) {
        match init {
            BufferInitContent::Data(data) => (
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: self.label,
                        usage: self.usage,
                        contents: data,
                    }),
                data.len() as u64,
            ),
            BufferInitContent::Size(size) => (
                self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: self.label,
                    size,
                    usage: self.usage,
                    mapped_at_creation: false,
                }),
                size,
            ),
        }
    }

    /// Creates the buffer
    #[must_use]
    pub fn create<T>(&self, contents: &[T]) -> Buffer
    where
        T: Pod,
    {
        let (inner, size) =
            self.create_impl(BufferInitContent::Data(bytemuck::cast_slice(contents)));

        Buffer {
            inner,
            gpu: self.gpu.clone(),
            size,
        }
    }

    /// Builds a buffer with 0s, with size in bytes
    #[must_use]
    pub fn create_uninit(&self, size: u64) -> Buffer {
        let (inner, size) = self.create_impl(BufferInitContent::Size(size));

        Buffer {
            inner,
            gpu: self.gpu.clone(),
            size,
        }
    }
}
