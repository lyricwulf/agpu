use crate::Gpu;

mod builder;
pub use builder::*;

pub struct RenderPipeline {
    pub gpu: Gpu,
    pub inner: wgpu::RenderPipeline,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
}
impl RenderPipeline {
    pub fn new(
        gpu: Gpu,
        inner: wgpu::RenderPipeline,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        Self {
            gpu,
            inner,
            depth_stencil,
        }
    }
}
crate::wgpu_inner_deref!(RenderPipeline);

pub trait Renderer {
    fn render();
}
