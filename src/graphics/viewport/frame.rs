use crate::{GpuError, GpuHandle, RenderPassBuilder};
use std::mem::ManuallyDrop;

/// Convenience wrapper for a frame buffer you render to.
/// `Frame` will automatically submit the finished encoder to the queue
/// and present the frame to the screen.
/// This was previously a repesentation of the swap chain
/// but since wgpu 0.11 it now wraps the surface texture
pub struct Frame<'a> {
    /// The gpu handle is ref'd because of the short lifetime of Frame
    pub(crate) gpu: &'a GpuHandle,
    /// The surface texture provided by the surface
    /// ManuallyDrop because we call `.present()` on it to present to screen
    surface_texture: ManuallyDrop<wgpu::SurfaceTexture>,
    /// The Optional depth texture
    depth_texture: Option<wgpu::TextureView>,
    pub view: wgpu::TextureView,
    pub encoder: ManuallyDrop<wgpu::CommandEncoder>,
    pub delta_time: Option<f32>,
}

impl Frame<'_> {
    pub fn render_pass<'f>(&'f mut self, label: &'f str) -> RenderPassBuilder {
        RenderPassBuilder {
            encoder: &mut self.encoder,
            gpu: self.gpu,
            desc: wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[],
                // TODO: This will map to the surface depth
                depth_stencil_attachment: self.depth_texture.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                }),
            },
            attachments: vec![wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            init_pipeline: None,
        }
    }
}

impl<'a> Frame<'a> {
    /// Creates a new Frame from the Surface
    /// We
    pub fn new(gpu: &'a GpuHandle, surface: &wgpu::Surface) -> Result<Self, GpuError> {
        let frame = surface
            .get_current_texture()
            .map_err(GpuError::SurfaceError)?;
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            // TODO: Custom label
            label: Some("Viewport frame view"),
            ..Default::default()
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Viewport render encoder"),
            });

        gpu.begin_profiler_section("Frame start", &mut encoder);

        Ok(Frame {
            gpu,
            surface_texture: ManuallyDrop::new(frame),
            depth_texture: None,
            view: frame_view,
            encoder: ManuallyDrop::new(encoder),
            delta_time: None,
        })
    }
}

/// On drop, we submit the encoder to the queue and present the frame
impl Drop for Frame<'_> {
    fn drop(&mut self) {
        // Take ownership of the encoder field so that it can be consumed by submit()
        // and the frame field so it can be consumed by present()
        // This is safe because we are dropping the struct right after this
        let encoder = unsafe { ManuallyDrop::take(&mut self.encoder) };
        let frame = unsafe { ManuallyDrop::take(&mut self.surface_texture) };

        // First submit the encoder to the queue
        self.gpu.queue.submit(Some(encoder.finish()));
        // Then present the frame to the screen
        frame.present();
        self.gpu.profiler.clear();
    }
}
