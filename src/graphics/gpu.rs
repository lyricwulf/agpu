mod builder;
pub use builder::GpuBuilder;

pub use wgpu::Backends;

use crate::{BufferBuilder, GpuError, Profiler, ViewportBuilder};
use std::{ops::Deref, rc::Rc};
use winit::window::Window;

/// The HW GPU context which contains all wgpu context info.
/// This is meant as an easier and more ergonomic way to pass around wgpu info.
/// You can manually construct this with fields but it is recommended to use the [builder].
///
/// [builder]: Gpu::builder()
pub struct Gpu {
    /// This is the instance for wgpu itself. We shouldn't need more than 1 in the
    /// life of a program.
    pub instance: wgpu::Instance,
    /// This is the adapter, representing the physical device.
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub profiler: Profiler,
}
impl Gpu {
    /// An alias for `GpuBuilder::new()`
    #[must_use]
    pub fn builder<'a>() -> GpuBuilder<'a> {
        GpuBuilder::default()
    }

    /// Converts the Gpu into a `GpuHandle` which can be passed around by clone
    #[must_use]
    pub fn to_handle(self) -> GpuHandle {
        GpuHandle {
            context: Rc::new(self),
        }
    }
}

/// A struct that wraps over `Rc<Gpu>` which can be passed around by clone.
/// Because this is a `Rc`, it will automatically be freed when there are no
/// more references to it. It follows that any struct with a `GpuHandle` will be
/// always be guaranteed a valid reference to the `Gpu`.
#[derive(Clone)]
pub struct GpuHandle {
    context: Rc<Gpu>,
}
impl GpuHandle {
    /// Create a Viewport for displaying to the given window.
    // Lifetime `a`: The reference Gpu and Window must outlive ViewportBuilder
    #[must_use]
    pub fn create_viewport(&self, window: Window) -> ViewportBuilder {
        ViewportBuilder::new(self.clone(), window)
    }

    #[must_use]
    pub fn create_buffer<'a>(&self, label: &'a str) -> BufferBuilder<'a> {
        BufferBuilder::new(self.clone(), label)
    }

    #[must_use]
    pub fn create_pipeline<'a>(&self) -> crate::pipeline::PipelineBuilder<'a> {
        crate::pipeline::PipelineBuilder::new(self.clone())
    }

    pub fn create_command_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    pub(crate) fn begin_profiler_section<'a>(
        &self,
        label: &str,
        encoder: &'a mut wgpu::CommandEncoder,
    ) {
        self.profiler.begin_section(label);
        self.profiler.timestamp(label, encoder);
    }

    pub(crate) fn begin_pipeline_statistics_query(&self, render_pass: &mut wgpu::RenderPass) {
        self.profiler.begin_stats(render_pass);
    }

    pub fn total_statistics(&self) -> Result<[u64; 5], GpuError> {
        // Get the QuerySet from the profiler
        let stats = self.profiler.stats.as_ref().ok_or(GpuError::QueryNone)?;

        let mut ret = [0; 5];

        for (i, stat) in stats
            .get(&self.device, self.profiler.query_count())?
            .iter()
            .enumerate()
        {
            ret[i % 5] += stat;
        }

        Ok(ret)
    }

    pub fn timestamp_report(&self) -> Vec<(String, f32)> {
        self.profiler.timestamp_report(&self.device)
    }
}

impl Deref for GpuHandle {
    type Target = Gpu;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}