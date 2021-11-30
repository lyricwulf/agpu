use crate::GpuHandle;

mod builder;
pub use builder::PipelineBuilder;

pub struct RenderPipeline {
    pub gpu: GpuHandle,
    inner: wgpu::RenderPipeline,
}
crate::wgpu_inner_deref!(RenderPipeline);

pub trait Renderer {
    fn render();
}
