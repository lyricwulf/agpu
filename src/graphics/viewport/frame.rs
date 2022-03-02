use crate::{CommandEncoder, Gpu, GpuError, RenderPassBuilder};
use std::mem::ManuallyDrop;

/// Convenience wrapper for a frame buffer you render to.
/// `Frame` will automatically submit the finished encoder to the queue
/// and present the frame to the screen.
/// This was previously a repesentation of the swap chain
/// but since wgpu 0.11 it now wraps the surface texture
pub struct Frame<'a> {
    /// The gpu handle is ref'd because of the short lifetime of Frame
    pub(crate) gpu: &'a Gpu,
    /// The surface texture provided by the surface
    /// ManuallyDrop because we call `.present()` on it to present to screen
    surface_texture: ManuallyDrop<wgpu::SurfaceTexture>,
    /// The Optional depth texture
    pub depth_texture: crate::TextureView<'a>,
    pub view: crate::TextureView<'a>,
    pub encoder: ManuallyDrop<CommandEncoder>,
    pub delta_time: Option<f32>,
    pub resized_to: Option<(u32, u32)>,
}

impl Frame<'_> {
    pub fn render_pass<'f>(&'f mut self, label: &'f str) -> RenderPassBuilder {
        RenderPassBuilder {
            encoder: &mut self.encoder,
            desc: wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[],
                // TODO: This will map to the surface depth
                depth_stencil_attachment: None,
            },
            init_color_attachments: Some(vec![wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }]),
            init_pipeline: None,
        }
    }

    pub fn render_pass_cleared<'f>(
        &'f mut self,
        label: &'f str,
        clear_color: u32,
    ) -> RenderPassBuilder {
        let [r, g, b, a] = clear_color.to_be_bytes();
        RenderPassBuilder {
            encoder: &mut self.encoder,
            desc: wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[],
                depth_stencil_attachment: None,
            },
            init_color_attachments: Some(vec![wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: r as f64 / 255.0,
                        g: g as f64 / 255.0,
                        b: b as f64 / 255.0,
                        a: a as f64 / 255.0,
                    }),
                    store: true,
                },
            }]),
            init_pipeline: None,
        }
    }

    pub fn create_encoder(&self, label: &str) -> CommandEncoder {
        self.gpu.create_command_encoder(label)
    }
}

impl<'a> Frame<'a> {
    /// Creates a new Frame from the Surface
    /// We
    pub fn new(
        gpu: &'a Gpu,
        surface: &wgpu::Surface,
        depth: wgpu::TextureView,
    ) -> Result<Self, GpuError> {
        let frame = surface
            .get_current_texture()
            .map_err(GpuError::SurfaceError)?;
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            // TODO: Custom label
            label: Some("Viewport frame view"),
            ..Default::default()
        });
        let mut encoder = gpu.create_command_encoder("Viewport render encoder");

        gpu.begin_profiler_section("Frame start", &mut encoder);

        Ok(Frame {
            gpu,
            surface_texture: ManuallyDrop::new(frame),
            depth_texture: gpu.wrap_view(depth),
            view: gpu.wrap_view(frame_view),
            encoder: ManuallyDrop::new(encoder),
            delta_time: None,
            resized_to: None,
        })
    }
}

/// On drop, we submit the encoder to the queue and present the frame
impl Drop for Frame<'_> {
    fn drop(&mut self) {
        // Take ownership of the encoder field so that it can be consumed by submit()
        // and the frame field so it can be consumed by present()
        // This is safe because we are dropping the struct right after this
        let frame = unsafe { ManuallyDrop::take(&mut self.surface_texture) };

        // First submit the encoder to the queue
        unsafe { ManuallyDrop::drop(&mut self.encoder) };
        // Then present the frame to the screen
        frame.present();
        self.gpu.profiler.clear();
    }
}
