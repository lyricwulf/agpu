pub use winit;
use winit::event_loop::ControlFlow;

use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

use crate::{Frame, GpuBuilder, GpuError, GpuHandle, Viewport};

mod display;

/// A stateful opaque type that handles framerate and timing for continuous
/// rendering.
pub struct ProgramTime {
    pub target_framerate: f32,
    pub last_update_time: Cell<Option<Instant>>,
    pub last_frame_time: Cell<Duration>,
    pub delta_time: Cell<Duration>,
    // Amount to shorten the frame time by, to make up for redraw dispatch time
    frame_time_adjustment: Cell<Duration>,
    // The last time the frame was drawn. Used for calculating the adjusted frame time
    last_draw_time: Cell<Option<Instant>>,
}
impl ProgramTime {
    pub fn new(framerate: f32) -> Self {
        Self {
            target_framerate: framerate,
            // Init last update time to zero'd for performance
            last_update_time: Cell::new(None),
            last_frame_time: Cell::new(Duration::ZERO),
            delta_time: Cell::new(Duration::ZERO),
            frame_time_adjustment: Cell::new(Duration::ZERO),
            last_draw_time: Cell::new(None),
        }
    }

    pub fn should_draw(&self) -> bool {
        // Note that this is the only time where we get now() to minmize the runtime cost
        let now = Instant::now();

        // Calculate the target frame time based on the framerate
        let framerate_time = Duration::from_secs_f32(1.0 / self.target_framerate);

        // Adjust the target frame time based on the last delta time
        let target_frametime = framerate_time.saturating_sub(self.frame_time_adjustment.get());

        // * Use let-else (RFC 3137) when available
        let last_update_time = if let Some(last_update_time) = self.last_update_time.get() {
            last_update_time
        } else {
            // Handle first update without time compensation
            self.last_update_time.set(Some(now));
            return true;
        };

        // Calculate the time since the last frame
        let time_since_last_frame = now - last_update_time;
        // Calculate adjusted wait time
        if let Some(last_draw_time) = self.last_draw_time.take() {
            let _last_draw_duration = now - last_draw_time;
            let new_adjustment = self.delta_time.get().saturating_sub(framerate_time);
            self.frame_time_adjustment.set(new_adjustment);
        }

        // Render a new frame if it has been long enough
        if time_since_last_frame >= target_frametime {
            self.last_update_time.set(Some(now));
            self.last_draw_time.set(Some(now));
            self.delta_time.set(time_since_last_frame);

            return true;
        }
        false
    }

    /// Clear the accumulated frame timer
    pub fn clear_counter(&self, now: Instant) {
        self.delta_time
            .set(now - self.last_update_time.get().unwrap_or(now));
        self.last_update_time.set(Some(now));
        self.last_draw_time.set(Some(now));
    }
}

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
    pub time: Option<ProgramTime>,
}

type ResizeFn = Box<dyn FnMut(&GpuProgram, u32, u32)>;

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
        self.run(move |event, _, _, _| {
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
                &Self,
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
                    } => *control_flow = ControlFlow::Exit,

                    // Resize the viewport when the window is resized
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::Resized(new_size),
                        ..
                    } => {
                        let (width, height) = new_size.into();
                        self.viewport.resize(width, height);
                        let mut on_resize = self.on_resize.borrow_mut();
                        if let Some(handler) = on_resize.as_mut() {
                            handler(&self, width, height);
                        }
                    }

                    winit::event::Event::RedrawEventsCleared => {
                        // Clamp to some max framerate if the target framerate is set
                        if let Some(program_time) = &self.time {
                            if program_time.should_draw() {
                                self.viewport.request_redraw()
                            }
                        }
                    }

                    // Handle redrawing
                    // We manually call the event handler. This returns out of the closure iteration
                    winit::event::Event::RedrawRequested(w) => {
                        let resized_to = if let Some(new_size) = *self.viewport.resize_to.borrow() {
                            event_handler(Event::Resize(new_size), &self, event_loop, control_flow);
                            Some(new_size)
                        } else {
                            None
                        };

                        // We first call the event handler with the original event,
                        // in case the user wants to perform some operations before creating the Frame
                        event_handler(
                            Event::Winit(winit::event::Event::RedrawRequested(w)),
                            &self,
                            event_loop,
                            control_flow,
                        );

                        // Then we create a new Frame and pass it to the event handler
                        let mut frame = match self.viewport.begin_frame() {
                            Ok(mut frame) => {
                                frame.delta_time = self
                                    .time
                                    .as_ref()
                                    .map(|time| time.delta_time.get().as_secs_f32());
                                frame
                            }
                            Err(e) => {
                                // Rudimentary error handling. Just logs and continues
                                tracing::warn!("Requested frame but {}. Redraw is suppressed", e);
                                return;
                            }
                        };
                        frame.resized_to = resized_to;

                        // Then we call the event handler with the frame
                        event_handler(Event::RedrawFrame(frame), &self, event_loop, control_flow);

                        // Return early so we don't call the event handler twice (once from 2 above)
                        return;
                    }
                    _ => {}
                }
                let event = Event::Winit(event);
                event_handler(event, &self, event_loop, control_flow);
            })
    }

    pub fn on_resize(&self, handler: impl FnMut(&GpuProgram, u32, u32) + 'static) {
        let mut on_resize = self.on_resize.borrow_mut();
        *on_resize = Some(Box::new(handler));
    }

    pub fn set_framerate(&mut self, target_framerate: f32) {
        self.time = Some(ProgramTime::new(target_framerate));
    }

    pub fn current_monitor_max_framerate(&self) -> f32 {
        const DEFAULT_FRAMERATE: f32 = 60.0;

        if let Some(monitor) = self.viewport.window.current_monitor() {
            // find the max framerate out of all possible video modes
            if let Some(video_mode) = monitor
                .video_modes()
                .max_by_key(winit::monitor::VideoMode::refresh_rate)
            {
                return video_mode.refresh_rate() as f32;
            }
        }
        // if monitor is not found, or if list of video modes is empty, go back to default
        DEFAULT_FRAMERATE
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
    pub framerate: Option<f32>,
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

    /// Sets the window to continuously draw at the given framerate.
    pub fn with_framerate(mut self, framerate: f32) -> Self {
        self.framerate = Some(framerate);
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

        // Create time module if there is a target framerate
        let time = self.framerate.map(ProgramTime::new);

        Ok(GpuProgram {
            event_loop: Cell::new(Some(event_loop)),
            viewport,
            gpu,
            on_resize: RefCell::new(None),
            time,
        })
    }
}
