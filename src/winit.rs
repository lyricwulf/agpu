pub use winit;

use std::cell::{Cell, RefCell};

use crate::{Frame, GpuBuilder, GpuError, GpuHandle, Viewport};

mod display;

/// A window, gpu, and viewport all in one
// TODO: Add custom user events via EventLoop<T>
pub struct GpuProgram {
    /// # Event Loop
    /// This is a container for the winit event loop.
    /// Since event_loop.run() takes ownership of the event_loop,
    /// we wrap it in Option to allow for transfer of ownership
    /// and Cell to allow for interior mutability.
    pub event_loop: Cell<Option<winit::event_loop::EventLoop<()>>>,
    pub gpu: GpuHandle,
    pub viewport: Viewport,
    pub on_resize: RefCell<Option<ResizeFn>>,
}

type ResizeFn = Box<dyn Fn(&GpuProgram, u32, u32)>;

impl GpuProgram {
    pub fn builder<'f>(title: &str) -> GpuProgramBuilder<'f> {
        GpuProgramBuilder::new().with_title(title)
    }

    /// Calls the closure each time a new frame is available to draw.
    /// This can be used to draw to the screen when you don't need to handle other events.
    pub fn run_draw<F>(self, mut op: F) -> !
    where
        F: 'static + FnMut(Frame<'_>),
    {
        self.run(move |event, _, _| {
            if let Event::RedrawFrame(frame) = event {
                op(frame);
            }
        })
    }

    /// Run the program
    /// This is a wrapper over Run that does some stuff automatically like resizing
    /// and closing the window
    /// # Panics
    /// Panics if run() was already used, as the event_loop is consumed on run
    /// This uses custom Event enum to allow for pass frame to redraw directly
    pub fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static
            + FnMut(
                Event<'_, ()>,
                &winit::event_loop::EventLoopWindowTarget<()>,
                &mut winit::event_loop::ControlFlow,
            ),
    {
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, event_loop, control_flow| {
                match event {
                    // Exit when close is requested
                    // This might not always be desired but it's fine for now.
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,

                    // Resize the viewport when the window is resized
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::Resized(new_size),
                        ..
                    } => {
                        let (width, height) = new_size.into();
                        self.viewport.resize(width, height);
                        let on_resize = self.on_resize.borrow();
                        if let Some(handler) = on_resize.as_ref() {
                            handler(&self, width, height);
                        }
                    }

                    // The state has been changed. We assume the display should change,
                    // so we request a redraw
                    winit::event::Event::MainEventsCleared => {
                        self.viewport.request_redraw();
                    }

                    // Handle redrawing
                    // We manually call the event handler. This returns out of the closure iteration
                    winit::event::Event::RedrawRequested(w) => {
                        if let Some(new_size) = *self.viewport.resize_to.borrow() {
                            event_handler(Event::Resize(new_size), event_loop, control_flow);
                        };

                        // We first call the event handler with the original event,
                        // in case the user wants to perform some operations before creating the Frame
                        event_handler(
                            Event::Winit(winit::event::Event::RedrawRequested(w)),
                            event_loop,
                            control_flow,
                        );

                        // Then we create a new Frame and pass it to the event handler
                        let frame = match self.viewport.begin_frame() {
                            Ok(frame) => frame,
                            Err(e) => {
                                // Rudimentary error handling. Just logs and continues
                                tracing::warn!("Requested frame but {}. Redraw is suppressed", e);
                                return;
                            }
                        };

                        // Then we call the event handler with the frame
                        event_handler(Event::RedrawFrame(frame), event_loop, control_flow);

                        // Return early so we don't call the event handler twice (once from 2 above)
                        return;
                    }
                    _ => {}
                }
                let event = Event::Winit(event);
                event_handler(event, event_loop, control_flow);
            })
    }

    pub fn on_resize(&self, handler: impl Fn(&GpuProgram, u32, u32) + 'static) {
        let mut on_resize = self.on_resize.borrow_mut();
        *on_resize = Some(Box::new(handler));
    }
}

pub enum Event<'a, T: 'static> {
    Winit(winit::event::Event<'a, T>),
    RedrawFrame(Frame<'a>),
    /// Called before resolving a pending resize.
    /// This is different from winit's ResizeRequested.
    /// This is only called right before drawing, so there is a promise that the
    /// window will not be further resized before the next frame is drawn.
    Resize((u32, u32)),
}

#[derive(Default)]
pub struct GpuProgramBuilder<'a> {
    pub window: winit::window::WindowBuilder,
    pub gpu: GpuBuilder<'a>,
}

impl<'a> GpuProgramBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_gpu_features(mut self, features: wgpu::Features) -> Self {
        self.gpu = self.gpu.with_features(features);
        self
    }

    // We simply reimplement the winit window builder's methods

    // ! WINIT BUILDER METHODS
    // FIXME: Please, there HAS to be a better way to do this

    /// Requests the window to be of specific dimensions.
    ///
    /// See [`Window::set_inner_size`] for details.
    ///
    /// [`Window::set_inner_size`]: crate::window::Window::set_inner_size
    #[inline]
    pub fn with_inner_size<S: Into<winit::dpi::Size>>(mut self, size: S) -> Self {
        self.window.window.inner_size = Some(size.into());
        self
    }

    /// Sets a minimum dimension size for the window.
    ///
    /// See [`Window::set_min_inner_size`] for details.
    ///
    /// [`Window::set_min_inner_size`]: crate::window::Window::set_min_inner_size
    #[inline]
    pub fn with_min_inner_size<S: Into<winit::dpi::Size>>(mut self, min_size: S) -> Self {
        self.window.window.min_inner_size = Some(min_size.into());
        self
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// See [`Window::set_max_inner_size`] for details.
    ///
    /// [`Window::set_max_inner_size`]: crate::window::Window::set_max_inner_size
    #[inline]
    pub fn with_max_inner_size<S: Into<winit::dpi::Size>>(mut self, max_size: S) -> Self {
        self.window.window.max_inner_size = Some(max_size.into());
        self
    }

    /// Sets a desired initial position for the window.
    ///
    /// See [`WindowAttributes::position`] for details.
    ///
    /// [`WindowAttributes::position`]: crate::window::WindowAttributes::position
    #[inline]
    pub fn with_position<P: Into<winit::dpi::Position>>(mut self, position: P) -> Self {
        self.window.window.position = Some(position.into());
        self
    }

    /// Sets whether the window is resizable or not.
    ///
    /// See [`Window::set_resizable`] for details.
    ///
    /// [`Window::set_resizable`]: crate::window::Window::set_resizable
    #[inline]
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.window.window.resizable = resizable;
        self
    }

    /// Requests a specific title for the window.
    ///
    /// See [`Window::set_title`] for details.
    ///
    /// [`Window::set_title`]: crate::window::Window::set_title
    #[inline]
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.window.window.title = title.into();
        self
    }

    /// Sets the window fullscreen state.
    ///
    /// See [`Window::set_fullscreen`] for details.
    ///
    /// [`Window::set_fullscreen`]: crate::window::Window::set_fullscreen
    #[inline]
    pub fn with_fullscreen(mut self, fullscreen: winit::window::Fullscreen) -> Self {
        self.window.window.fullscreen = Some(fullscreen);
        self
    }

    /// Requests maximized mode.
    ///
    /// See [`Window::set_maximized`] for details.
    ///
    /// [`Window::set_maximized`]: crate::window::Window::set_maximized
    #[inline]
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.window.window.maximized = maximized;
        self
    }

    /// Sets whether the window will be initially hidden or visible.
    ///
    /// See [`Window::set_visible`] for details.
    ///
    /// [`Window::set_visible`]: crate::window::Window::set_visible
    #[inline]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.window.window.visible = visible;
        self
    }

    /// Sets whether the background of the window should be transparent.
    #[inline]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.window.window.transparent = transparent;
        self
    }

    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// See [`Window::set_decorations`] for details.
    ///
    /// [`Window::set_decorations`]: crate::window::Window::set_decorations
    #[inline]
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.window.window.decorations = decorations;
        self
    }

    /// Sets whether or not the window will always be on top of other windows.
    ///
    /// See [`Window::set_always_on_top`] for details.
    ///
    /// [`Window::set_always_on_top`]: crate::window::Window::set_always_on_top
    #[inline]
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.window.window.always_on_top = always_on_top;
        self
    }

    /// Sets the window icon.
    ///
    /// See [`Window::set_window_icon`] for details.
    ///
    /// [`Window::set_window_icon`]: crate::window::Window::set_window_icon
    #[inline]
    pub fn with_window_icon(mut self, window_icon: winit::window::Icon) -> Self {
        self.window.window.window_icon = Some(window_icon);
        self
    }

    pub fn build(self) -> Result<GpuProgram, GpuError> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = self.window.build(&event_loop).unwrap();

        let gpu = self.gpu.clone();
        let gpu = gpu.with_profiler().build(&window)?;
        let viewport = gpu.new_viewport(window).create();

        Ok(GpuProgram {
            event_loop: Cell::new(Some(event_loop)),
            viewport,
            gpu,
            on_resize: RefCell::new(None),
        })
    }
}
