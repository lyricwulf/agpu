use std::ops::{Deref, DerefMut};

use crate::{CommandEncoder, Frame, GpuHandle, RenderPipeline, Texture};

pub struct RenderPassBuilder<'a, 'b> {
    /// Encoder is used to create the render pass on build()
    pub(crate) encoder: &'a mut CommandEncoder,
    // /// Reference to the profiler we will submit to
    // pub(crate) gpu: &'a crate::GpuHandle,
    pub(crate) desc: wgpu::RenderPassDescriptor<'a, 'b>,
    pub(crate) init_color_attachments: Option<Vec<wgpu::RenderPassColorAttachment<'a>>>,
    /// An optional pipeline that the render pass will start with
    /// This is ergonomic for single-pipeline render passes,
    /// but it is fairly useless otherwise
    pub(crate) init_pipeline: Option<&'a wgpu::RenderPipeline>,
}

pub trait RenderAttachmentBuild {
    fn clear_impl(self, r: f64, g: f64, b: f64, a: f64) -> Self;
    fn clear(self) -> Self;
    fn clear_black(self) -> Self;
    fn clear_white(self) -> Self;
    fn clear_color(self, color: u32) -> Self;
    fn readonly(self) -> Self;
}

impl<'a> RenderAttachmentBuild for wgpu::RenderPassColorAttachment<'a> {
    fn clear_impl(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
            ..self.ops
        };
        self
    }
    fn clear(self) -> Self {
        self.clear_impl(0.0, 0.0, 0.0, 0.0)
    }
    fn clear_black(self) -> Self {
        self.clear_impl(0.0, 0.0, 0.0, 1.0)
    }
    fn clear_white(self) -> Self {
        self.clear_impl(1.0, 1.0, 1.0, 1.0)
    }
    fn clear_color(self, color: u32) -> Self {
        let [r, g, b, a] = color.to_be_bytes();
        self.clear_impl(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        )
    }

    fn readonly(mut self) -> Self {
        self.ops = wgpu::Operations {
            store: false,
            ..self.ops
        };
        self
    }
}

pub trait DepthAttachmentBuild {
    fn clear_depth_val(self, val: f32) -> Self;
    fn clear_stencil_val(self, val: u32) -> Self;
    fn clear_depth(self) -> Self;
    fn clear_stencil(self) -> Self;
    fn clear(self) -> Self;
}

impl<'a> DepthAttachmentBuild for wgpu::RenderPassDepthStencilAttachment<'a> {
    fn clear_depth_val(mut self, val: f32) -> Self {
        if let Some(mut ops) = self.depth_ops.as_mut() {
            ops.load = wgpu::LoadOp::Clear(val);
        }
        self
    }
    fn clear_stencil_val(mut self, val: u32) -> Self {
        if let Some(mut ops) = self.stencil_ops.as_mut() {
            ops.load = wgpu::LoadOp::Clear(val);
        }
        self
    }
    /// Clear depth to 1.0
    fn clear_depth(self) -> Self {
        // Depth is usually cleared to 1.0
        self.clear_depth_val(1.0)
    }
    /// Clear stencil to 0
    fn clear_stencil(self) -> Self {
        self.clear_stencil_val(0)
    }
    /// Clear depth to 1.0 and stencil to 0, if they exist respectively
    fn clear(self) -> Self {
        self.clear_depth().clear_stencil()
    }
}

impl<'a, 'b> RenderPassBuilder<'a, 'b> {
    pub fn new(encoder: &'a mut CommandEncoder, _gpu: &'a mut GpuHandle) -> Self {
        Self {
            encoder,
            desc: wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[],
                depth_stencil_attachment: None,
            },
            init_pipeline: None,
            init_color_attachments: None,
        }
    }

    pub fn with_depth(mut self, depth: wgpu::RenderPassDepthStencilAttachment<'a>) -> Self {
        self.desc.depth_stencil_attachment = Some(depth);
        self
    }

    #[inline]
    pub fn with_pipeline(mut self, pipeline: &'a RenderPipeline) -> Self {
        self.init_pipeline = Some(pipeline);
        if pipeline.depth_stencil.is_none() {
            self.desc.depth_stencil_attachment = None;
        }
        self
    }

    pub fn begin(self) -> RenderPass<'a> {
        let desc = self.desc.clone();
        self.begin_impl(&desc)
    }

    fn begin_impl(self, desc: &'b wgpu::RenderPassDescriptor<'a, 'b>) -> RenderPass<'a> {
        let gpu = self.encoder.gpu.clone();
        if self.encoder.gpu.profiler.timestamp.is_some() {
            gpu.begin_profiler_section(self.desc.label.unwrap_or("Render pass"), self.encoder);
        }

        // Create the inner render pass
        // Use the init attachment if it exists
        let mut inner = if let Some(init_attachment) = self.init_color_attachments {
            let color_attachments = &init_attachment;
            let desc = wgpu::RenderPassDescriptor {
                color_attachments,
                ..desc.clone()
            };
            self.encoder.begin_render_pass(&desc)
        } else {
            self.encoder.begin_render_pass(desc)
        };

        let pipeline_statistics = gpu.profiler.stats.is_some();

        if pipeline_statistics {
            gpu.begin_pipeline_statistics_query(&mut inner);
        }

        if let Some(pipeline) = self.init_pipeline {
            inner.set_pipeline(pipeline);
        }

        RenderPass {
            inner,
            pipeline_statistics,
        }
    }
}

pub struct RenderPass<'a> {
    inner: wgpu::RenderPass<'a>,
    pipeline_statistics: bool,
}

impl RenderPass<'_> {
    pub fn draw_triangles(&mut self, count: u32) {
        self.inner.draw(0..3, 0..count);
    }

    #[inline]
    pub fn draw_triangle(&mut self) {
        self.inner.draw(0..3, 0..1);
    }

    pub fn draw_one(&mut self, vertices: u32) {
        self.inner.draw(0..vertices, 0..1);
    }

    pub fn draw_one_indexed(&mut self, vertices: u32) {
        self.inner.draw_indexed(0..vertices, 0, 0..1);
    }
}
impl<'a> RenderPass<'a> {
    /// Shadows wgpu::RenderPass::set_bind_group and returns self for chaining
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    ) -> &mut Self {
        self.inner.set_bind_group(index, bind_group, offsets);
        self
    }

    /// Shadows wgpu::RenderPass::set_pipeline and returns self for chaining
    pub fn set_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline) -> &mut Self {
        self.inner.set_pipeline(pipeline);
        self
    }

    /// Shadows wgpu::RenderPass::set_index_buffer and returns self for chaining
    /// ### USES U16 FORMAT
    /// See `set_index_buffer_u32` for a version that uses u32
    pub fn set_index_buffer(&mut self, buffer_slice: wgpu::BufferSlice<'a>) -> &mut Self {
        self.inner
            .set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint16);
        self
    }

    /// See `set_index_buffer`
    pub fn set_index_buffer_u32(&mut self, buffer_slice: wgpu::BufferSlice<'a>) -> &mut Self {
        self.inner
            .set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint32);
        self
    }

    /// Shadows wgpu::RenderPass::set_vertex_buffer and returns self for chaining
    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_slice: wgpu::BufferSlice<'a>,
    ) -> &mut Self {
        self.inner.set_vertex_buffer(slot, buffer_slice);
        self
    }

    /// Sets the scissor region.
    ///
    /// Subsequent draw calls will discard any fragments that fall outside this region.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) -> &mut Self {
        self.inner.set_scissor_rect(x, y, width, height);
        self
    }
}

impl<'a> Deref for RenderPass<'a> {
    type Target = wgpu::RenderPass<'a>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'a> DerefMut for RenderPass<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> From<wgpu::RenderPass<'a>> for RenderPass<'a> {
    fn from(render_pass: wgpu::RenderPass<'a>) -> Self {
        Self {
            inner: render_pass,
            pipeline_statistics: false,
        }
    }
}

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        if self.pipeline_statistics {
            self.end_pipeline_statistics_query();
        }
    }
}

impl crate::CommandEncoder {
    pub fn render_pass<'a, 'b>(
        &'a mut self,
        label: &'a str,
        targets: &'b [wgpu::RenderPassColorAttachment<'a>],
    ) -> RenderPassBuilder<'a, 'b> {
        RenderPassBuilder {
            encoder: self,
            desc: wgpu::RenderPassDescriptor {
                color_attachments: targets,
                label: Some(label),
                depth_stencil_attachment: None,
            },
            init_pipeline: None,
            init_color_attachments: None,
        }
    }
}

impl Texture {
    pub fn attach_render(&self) -> RenderAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    }

    pub fn attach_depth(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
        }
    }

    pub fn attach_stencil(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
        }
    }

    pub fn attach_depth_stencil(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
        }
    }
}

pub type RenderAttachment<'a> = wgpu::RenderPassColorAttachment<'a>;
pub type DepthAttachment<'a> = wgpu::RenderPassDepthStencilAttachment<'a>;

impl Frame<'_> {
    pub const fn attach_render(&self) -> RenderAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    }

    pub const fn attach_depth(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
        }
    }

    pub const fn attach_stencil(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture,
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
        }
    }

    pub const fn attach_depth_stencil(&self) -> DepthAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
        }
    }
}
