mod view;
pub use view::*;

mod sampler;
pub use sampler::*;

mod builder;
pub use builder::*;

pub struct Texture {
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
}
crate::wgpu_inner_deref!(Texture);
