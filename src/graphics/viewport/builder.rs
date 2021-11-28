use crate::{GpuHandle, Viewport};
use winit::window::Window;

pub struct ViewportBuilder {
    pub gpu: GpuHandle,
    pub window: Window,
    pub format: Option<wgpu::TextureFormat>,
}
impl<'a> ViewportBuilder {
    pub fn new(gpu: GpuHandle, window: Window) -> Self {
        Self {
            gpu,
            window,
            format: None,
        }
    }

    pub fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Build the Viewport.
    /// Note this builder is consumed to pass the GpuHandle to the built Viewport.
    pub fn create(self) -> Viewport {
        let size = self.window.inner_size();
        let surface = unsafe { self.gpu.instance.create_surface(&self.window) };
        let format = if let Some(format) = self.format {
            format
        } else {
            surface.get_preferred_format(&self.gpu.adapter).unwrap()
        };

        Viewport::new(
            self.gpu,
            surface,
            size.width,
            size.height,
            format,
            self.window,
        )
    }
}
