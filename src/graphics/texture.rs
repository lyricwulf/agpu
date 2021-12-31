mod view;
pub use view::*;

mod sampler;
pub use sampler::*;

mod builder;
pub use builder::*;

use crate::GpuHandle;

// Re-export TextureFormat
pub use wgpu::TextureFormat;

pub struct Texture {
    pub(crate) gpu: GpuHandle,
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
    pub usage: wgpu::TextureUsages,
}
crate::wgpu_inner_deref!(Texture);
impl Texture {
    pub fn new(gpu: GpuHandle, desc: &wgpu::TextureDescriptor) -> Self {
        let inner = gpu.create_texture(desc);
        let view = inner.create_view(&Default::default());
        Self {
            gpu,
            inner,
            view,
            format: desc.format,
            size: desc.size,
            usage: desc.usage,
        }
    }

    pub fn resize(&mut self, size: &[u32]) {
        let (width, height, depth_or_array_layers) = {
            let mut size = size.iter();
            (
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
            )
        };
        let new_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let new_usage = self.usage | wgpu::TextureUsages::COPY_DST;

        let new_texture = self.gpu.create_texture(&wgpu::TextureDescriptor {
            // TODO: update label for texture on resize
            label: None,
            size: new_size,
            // TODO: mip level count on resize??
            mip_level_count: 1,
            sample_count: 1,
            dimension: builder::size_dim(&self.size),
            format: self.format,
            usage: new_usage,
        });
        let mut enc = self.gpu.create_command_encoder("Texture resize encoder");
        enc.copy_texture_to_texture(
            self.inner.as_image_copy(),
            new_texture.as_image_copy(),
            self.size,
        );
        self.gpu.queue.submit([enc.finish()]);

        self.inner = new_texture;
        self.size = new_size;
        self.usage = new_usage;
    }
}
