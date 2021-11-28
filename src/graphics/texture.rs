mod view;
pub use view::*;

mod sampler;
pub use sampler::*;

mod builder;
pub use builder::*;

// Re-export TextureFormat
pub use wgpu::TextureFormat;

pub struct Texture {
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}
crate::wgpu_inner_deref!(Texture);
