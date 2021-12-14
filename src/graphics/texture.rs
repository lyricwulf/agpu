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
        }
    }

    pub fn resize(&mut self, _size: &[u32]) {
        self.gpu.create_texture(&wgpu::TextureDescriptor {
            label: Default::default(),
            size: Default::default(),
            mip_level_count: Default::default(),
            sample_count: Default::default(),
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::empty(),
        });
        todo!()
    }
}
