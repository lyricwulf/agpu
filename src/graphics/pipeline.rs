use crate::GpuHandle;

mod builder;
pub use builder::PipelineBuilder;

pub struct RenderPipeline {
    pub gpu: GpuHandle,
    pub inner: wgpu::RenderPipeline,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
}
impl RenderPipeline {
    pub fn new(
        gpu: GpuHandle,
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
