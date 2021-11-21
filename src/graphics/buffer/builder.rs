use bytemuck::Pod;
use wgpu::util::DeviceExt;

use tracing::warn;

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
    pub content: BufferInitContent<'a>,
    pub usage: wgpu::BufferUsages,
}
impl<'a> BufferBuilder<'a> {
    #[must_use]
    pub const fn new(gpu: GpuHandle, label: &'a str) -> Self {
        BufferBuilder {
            gpu,
            label: Some(label),
            content: BufferInitContent::Size(0),
            usage: wgpu::BufferUsages::empty(),
        }
    }

    /// Set a label that GPU debuggers can display
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// The buffer will be initialized with this size
    /// Mutually exclusive to `with_data`
    pub fn with_size(mut self, size: u64) -> Self {
        self.content = BufferInitContent::Size(size);
        self
    }

    /// The buffer will be initialized with the contents of the given slice
    /// Mutually exclusive to `with_size`
    pub fn with_data<T>(mut self, data: &'a [T]) -> Self
    where
        T: Pod,
    {
        self.content = BufferInitContent::Data(bytemuck::cast_slice(data));
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
    fn build_impl(&self, mapped_at_creation: bool) -> (wgpu::Buffer, u64) {
        match self.content {
            BufferInitContent::Data(data) => {
                if mapped_at_creation {
                    warn!("mapped a buffer on creation, but it is already being initialized with data");
                }
                (
                    self.gpu
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: self.label,
                            usage: self.usage,
                            contents: data,
                        }),
                    data.len() as u64,
                )
            }
            BufferInitContent::Size(size) => (
                self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: self.label,
                    size,
                    usage: self.usage,
                    mapped_at_creation,
                }),
                size,
            ),
        }
    }

    /// Creates the buffer
    #[must_use]
    pub fn build(&self) -> Buffer {
        let (inner, size) = self.build_impl(false);

        Buffer {
            inner,
            gpu: self.gpu.clone(),
            size,
        }
    }

    /// Allows a buffer to be mapped immediately after they are made.
    #[must_use]
    pub fn build_and_map(&self) -> Buffer {
        let (inner, size) = self.build_impl(true);

        Buffer {
            inner,
            gpu: self.gpu.clone(),
            size,
        }
    }
}
