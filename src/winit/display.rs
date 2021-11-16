use crate::{Frame, GpuError};

pub struct WindowView {
    pub window: winit::window::Window,
    pub viewport: crate::Viewport,
}

impl WindowView {
    fn resize_using_window(&self) -> bool {
        let (width, height) = self.window.inner_size().into();
        let changed = self.viewport.set_conf_size(width, height);
        if changed {
            self.viewport.resize_impl();
        }
        changed
    }
}

impl crate::RenderTarget for WindowView {
    fn begin_frame(&self) -> Result<Frame<'_>, GpuError> {
        match Frame::new(&self.viewport.gpu, &self.viewport.surface) {
            Ok(frame) => Ok(frame),
            Err(GpuError::SurfaceError(wgpu::SurfaceError::Outdated)) => {
                // Attempt to resize the window if the surface is outdated.
                // If the window is the same size, then a simple resize will
                // not solve this error.
                if self.resize_using_window() {
                    self.begin_frame()
                } else {
                    Err(GpuError::SurfaceError(wgpu::SurfaceError::Outdated))
                }
            }
            Err(e) => Err(e),
        }
    }
}
