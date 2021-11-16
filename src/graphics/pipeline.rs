use std::ops::Deref;

use crate::GpuHandle;

mod builder;
pub use builder::PipelineBuilder;

pub struct Pipeline {
    pub gpu: GpuHandle,
    inner: wgpu::RenderPipeline,
}

impl Deref for Pipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait RenderPipeline {
    fn render();
}
