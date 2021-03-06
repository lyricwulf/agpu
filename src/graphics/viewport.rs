mod builder;
pub use builder::*;

mod frame;
pub use frame::*;

mod render_pass;
pub use render_pass::*;

use std::{cell::RefCell, ops::Deref};

use crate::{bitor, Gpu, GpuError, Texture};

pub trait BeginRenderFrame {
    fn begin_frame(&self) -> Result<Frame, GpuError>;
}

/// A `Viewport` is a rectangular area of that can be presented.
// * Using RefCell for interior mutability is somewhat suboptimal since it does
// * have a runtime cost, but since we will not have many viewports or calls
// * to those RefCells, it should be fine.
// ? But should we have RefCell anyway? Maybe we should just use external mutability?
pub struct Viewport {
    pub gpu: Gpu,
    pub surface: wgpu::Surface,
    pub window: winit::window::Window,
    /// The swap chain descriptor contains the size and format of the swap chain texture
    /// Uses RefCell for interior mutability.
    pub sc_desc: RefCell<wgpu::SurfaceConfiguration>,
    /// Uses RefCell for interior mutability.
    // pub swap_chain: RefCell<wgpu::SwapChain>,
    pub depth_texture: RefCell<crate::Texture<crate::D2>>,
    /// Data buffer for viewport properties.
    /// Binding 0: viewport size f32x2
    pub data_buffer: wgpu::Buffer,
    /// A queued resize. Stored when resize() is called and applied before the next
    /// swapchain frame is given.
    /// Uses RefCell for interior mutability.
    pub resize_to: RefCell<Option<(u32, u32)>>,
}
impl<'a> Viewport {
    #[must_use]
    pub fn new(
        gpu: Gpu,
        surface: wgpu::Surface,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        window: winit::window::Window,
    ) -> Self {
        let sc_desc = wgpu::SurfaceConfiguration {
            usage: bitor!(
                wgpu::TextureUsages: COPY_SRC | COPY_DST | TEXTURE_BINDING | RENDER_ATTACHMENT
            ),
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&gpu.device, &sc_desc);

        let depth_texture = Self::create_depth_texture(&gpu, width, height);

        let data_buffer = gpu
            .new_buffer("Viewport buffer")
            .as_uniform_buffer()
            .allow_copy_to()
            .create(&[width as f32, height as f32])
            .inner;

        // Wrap in RefCell for interior mutability.
        let sc_desc = RefCell::new(sc_desc);
        // let swap_chain = RefCell::new(swap_chain);
        let depth_texture = RefCell::new(depth_texture);

        Self {
            gpu,
            surface,
            depth_texture,
            sc_desc,
            data_buffer,
            resize_to: RefCell::new(None),
            window,
        }
    }
    /// Create a swap chain with the inner swap chain descriptor.
    // fn create_swap_chain(&self) -> wgpu::SwapChain {
    //     return self
    //         .gpu
    //         .device
    //         .create_swap_chain(&self.surface, &self.sc_desc.borrow());
    // }

    /// Utility to create a swap chain and replace our swap chain with the new one.
    fn configure_surface(&self) {
        self.surface
            .configure(&self.gpu.device, &self.sc_desc.borrow());
    }

    fn create_depth_texture(gpu: &Gpu, width: u32, height: u32) -> Texture<crate::D2> {
        gpu.new_texture("Viewport depth texture")
            .as_render_target()
            .with_format(crate::TextureFormat::Depth32Float)
            .create_empty((width, height))
    }

    /// Queues a resize
    // fn resize_using_window(&self) -> bool {
    //     let (width, height) = self.window.inner_size().into();
    //     let resized = self.set_conf_size(width, height);
    //     self.resize_impl();
    //     resized
    // }

    pub(crate) fn set_conf_size(&self, width: u32, height: u32) -> bool {
        let mut sc_desc = self.sc_desc.borrow_mut();
        if sc_desc.width == width && sc_desc.height == height {
            return false;
        }
        sc_desc.width = width;
        sc_desc.height = height;
        true
    }

    /// Queue a resize to the given dimensions.
    /// This does not execute immediately, but will be applied before the next
    /// swapchain frame is given.
    pub fn resize(&self, width: u32, height: u32) {
        let mut resize_to = self.resize_to.borrow_mut();
        *resize_to = Some((width, height));
    }

    /// Performs a resize if one is queued.
    /// See `resize()`.
    fn resolve_resize(&self) {
        if let Some((width, height)) = self.resize_to.borrow_mut().take() {
            // Update the size in our struct
            self.set_conf_size(width, height);
            self.resize_impl();
        }
    }

    pub(crate) fn resize_impl(&self) {
        let sc_desc = &self.sc_desc.borrow();
        let (width, height) = (sc_desc.width, sc_desc.height);

        // Do not actually resize if the size is zero
        if width == 0 || height == 0 {
            return;
        }

        // Recreate the swap chain
        self.configure_surface();

        // depth
        let depth_texture = Self::create_depth_texture(&self.gpu, width, height);

        self.depth_texture.replace(depth_texture);

        // Update the data buffer
        self.gpu.queue.write_buffer(
            &self.data_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );
    }

    /// Get the next frame in the swap chain.
    /// # Errors
    /// Returns an error according to [`wgpu::SwapChainError`].
    #[deprecated(note = "Use begin_frame() instead.")]
    pub fn get_current_frame(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.resolve_resize();
        let texture = self.surface.get_current_texture()?;
        Ok(texture)
    }

    fn resize_using_window(&self) -> bool {
        let (width, height) = self.window.inner_size().into();
        let changed = self.set_conf_size(width, height);
        if changed {
            self.resize_impl();
        }
        changed
    }

    pub fn begin_frame(&self) -> Result<Frame, GpuError> {
        self.resolve_resize();

        // TODO: Ideally we should not be creating a new depth texture view every frame.
        match Frame::new(
            &self.gpu,
            &self.surface,
            self.depth_texture
                .borrow()
                .create_view(&wgpu::TextureViewDescriptor::default()),
        ) {
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

    #[allow(clippy::cast_precision_loss)]
    pub fn aspect_ratio(&self) -> f32 {
        let sc_desc = self.sc_desc.borrow();
        sc_desc.width as f32 / sc_desc.height as f32
    }

    pub fn width(&self) -> u32 {
        let sc_desc = self.sc_desc.borrow();
        sc_desc.width
    }

    pub fn height(&self) -> u32 {
        let sc_desc = self.sc_desc.borrow();
        sc_desc.height
    }

    /// Returns the area in pixels of the window.
    /// Useful to check if this is 0 for no drawing
    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }
}

impl Deref for Viewport {
    type Target = winit::window::Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
