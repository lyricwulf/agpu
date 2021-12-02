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

impl crate::BeginRenderFrame for WindowView {
    fn begin_frame(&self) -> Result<Frame<'_>, GpuError> {
        self.viewport.begin_frame()
    }
}
