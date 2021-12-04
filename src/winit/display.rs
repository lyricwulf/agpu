use crate::{Frame, GpuError};

pub struct WindowView {
    pub window: winit::window::Window,
    pub viewport: crate::Viewport,
}

impl crate::BeginRenderFrame for WindowView {
    fn begin_frame(&self) -> Result<Frame<'_>, GpuError> {
        self.viewport.begin_frame()
    }
}
