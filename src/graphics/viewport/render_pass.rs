use std::ops::{Deref, DerefMut};

use wgpu::LoadOp;

use crate::GpuHandle;

pub struct RenderPassBuilder<'a, 'b> {
    /// Encoder is used to create the render pass on build()
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    /// Reference to the profiler we will submit to
    pub(crate) gpu: &'a crate::GpuHandle,
    pub(crate) desc: wgpu::RenderPassDescriptor<'a, 'b>,
    pub(crate) attachments: Vec<wgpu::RenderPassColorAttachment<'a>>,
    /// An optional pipeline that the render pass will start with
    /// This is ergonomic for single-pipeline render passes,
    /// but it is fairly useless otherwise
    pub(crate) init_pipeline: Option<&'a wgpu::RenderPipeline>,
}

pub trait RenderAttachmentExt {
    fn clear(&mut self, value: u32) -> &Self;
    fn readonly(&mut self) -> &Self;
}

impl<'a> RenderAttachmentExt for wgpu::RenderPassColorAttachment<'a> {
    fn clear(&mut self, value: u32) -> &Self {
        let [r, g, b, a] = value.to_be_bytes();
        self.ops.load = LoadOp::Clear(wgpu::Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        });
        self
    }

    fn readonly(&mut self) -> &Self {
        self.ops.store = false;
        self
    }
}

// TODO: Allow multiple or custom render attachments
// * We do not always want to render directly to the swap chain
// * We may want to render to a texture, or a texture array

// pub struct RenderAttachmentBuilder<'a> {
//     view: &'a wgpu::TextureView,
//     /// Ignored when render attachment is is load mode.
//     /// A value of None means the attachment will load the previous frame's contents.
//     clear_value: Option<u32>,
//     store: bool,
// }

// impl<'a> RenderAttachmentBuilder<'a> {
//     pub fn as_color_attachment(&self) -> wgpu::RenderPassColorAttachment {
//         let load = if let Some(clear_value) = self.clear_value {
//             wgpu::LoadOp::Clear(wgpu::Color {
//                 r: (clear_value >> 24 & 0xFF) as f64 / 255.0,
//                 g: (clear_value >> 16 & 0xFF) as f64 / 255.0,
//                 b: (clear_value >> 8 & 0xFF) as f64 / 255.0,
//                 a: (clear_value >> 0 & 0xFF) as f64 / 255.0,
//             })
//         } else {
//             wgpu::LoadOp::Load
//         };

//         wgpu::Operations {
//             load,
//             store: self.store,
//         };
//         let ops = self.clear_value;

//         wgpu::RenderPassColorAttachment {
//             view: self.view,
//             resolve_target: None,
//             ops,
//         }
//     }

//     pub fn as_depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {}
// }

// pub trait RenderColorAttachment {
//     fn color_attachment(&self, attachment: &wgpu::Texture) -> &RenderAttachmentBuilder;
// }

impl<'a> RenderPassBuilder<'a, '_> {
    pub fn new(encoder: &'a mut wgpu::CommandEncoder, gpu: &'a mut GpuHandle) -> Self {
        Self {
            encoder,
            gpu,
            desc: wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[],
                depth_stencil_attachment: None,
            },
            attachments: vec![],
            init_pipeline: None,
        }
    }

    #[inline]
    pub fn with_pipeline(mut self, pipeline: &'a wgpu::RenderPipeline) -> Self {
        self.init_pipeline = Some(pipeline);
        self
    }

    // TODO: This does not scale for multiple render attachments
    pub fn clear_color(mut self, value: u32) -> Self {
        self.attachments[0].clear(value);
        self
    }

    pub fn begin(self) -> RenderPass<'a> {
        if self.gpu.profiler.timestamp.is_some() {
            self.gpu
                .begin_profiler_section(self.desc.label.unwrap_or("Render pass"), self.encoder);
        }

        let mut inner = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &self.attachments,
            ..self.desc
        });

        let pipeline_statistics = self.gpu.profiler.stats.is_some();

        if pipeline_statistics {
            self.gpu.begin_pipeline_statistics_query(&mut inner);
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
