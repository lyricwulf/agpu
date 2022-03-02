use futures::executor::block_on;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{
    graphics::{Gpu, GpuCtx, GpuError},
    Profiler,
};

#[derive(Clone)]
/// Builder for `GpuContext`.
/// By default this is initialized with sensible values for our use case.
pub struct GpuBuilder<'a> {
    /// The backends that wgpu should use.
    /// By default, this is only the PRIMARY backends, which have first-class support.
    /// You can alternatively specify individual backends such as `VULKAN` or `DX12`.
    backends: wgpu::Backends,
    /// The power preference for the adapter.
    /// This defaults to `HighPerformance` but can be set to use `LowPower`.
    power_preference: wgpu::PowerPreference,
    /// The device limits.
    limits: wgpu::Limits,
    /// The features that the device must support.
    features: wgpu::Features,
    /// The features that the device can optionally support.
    optional_features: wgpu::Features,
    /// The optional output trace path for wgpu
    trace_path: Option<&'a std::path::Path>,
    /// The label for this context.
    label: Option<&'a str>,
}
impl Default for GpuBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl GpuBuilder<'_> {
    /// Create a `GpuBuilder` with sensible defaults.
    pub fn new() -> Self {
        Self {
            backends: wgpu::Backends::PRIMARY,
            power_preference: wgpu::PowerPreference::HighPerformance,
            limits: wgpu::Limits::default(),
            label: None,
            features: wgpu::Features::default(),
            optional_features: wgpu::Features::empty(),
            trace_path: None,
        }
    }
}
impl<'a> GpuBuilder<'a> {
    /// Sets the backends that wgpu should use.
    pub fn with_backends(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    /// Sets the power preference for the adapter.
    pub fn with_power_preference(mut self, power_preference: wgpu::PowerPreference) -> Self {
        self.power_preference = power_preference;
        self
    }

    /// Sets the limits for the device.
    /// There is certainly a better way to do this.
    pub fn with_limits(mut self, limits: wgpu::Limits) -> Self {
        self.limits = limits;
        self
    }

    /// Sets the label for the device.
    /// Can be used with `&String` argument.
    /// The argument must outlive the builder.
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets the features that the device must support.
    pub fn with_features(mut self, features: wgpu::Features) -> Self {
        self.features = features;
        self
    }

    /// Sets the features that the device can optionally support.
    pub fn with_optional_features(mut self, features: wgpu::Features) -> Self {
        self.optional_features = features;
        self
    }

    /// Enables the GPU profiler by setting the corresponding feature flags
    pub fn with_profiler(mut self) -> Self {
        self.features |= wgpu::Features::TIMESTAMP_QUERY;
        self.features |= wgpu::Features::PIPELINE_STATISTICS_QUERY;
        self
    }

    /// Sets the output trace path for wgpu
    pub fn with_trace_path(mut self, path: &'a std::path::Path) -> Self {
        self.trace_path = Some(path);
        self
    }

    /// Shorthand for build_windowed_sync().
    /// # Errors
    /// Errors when the inner build() fails.
    pub fn build<W>(self, window: &W) -> Result<Gpu, GpuError>
    where
        W: HasRawWindowHandle,
    {
        block_on(self.build_impl(Some(window)))
    }

    pub fn build_headless(self) -> Result<Gpu, GpuError> {
        block_on(self.build_impl::<NoWindow>(None))
    }

    /// Build the `GpuContext` from the builder.
    /// Use `build_sync` for synchronous.
    /// # Errors
    /// Errors when a connection to the GPU could not be established.
    pub async fn build_impl<W>(self, window: Option<&W>) -> Result<Gpu, GpuError>
    where
        W: HasRawWindowHandle,
    {
        // Create the wgpu instance.
        let instance = wgpu::Instance::new(self.backends);

        // Create a surface to test compatibility, if there is a window.
        // Note that it is illegal to create a swapchain to the same surface twice,
        // however since we are only creating the surface and not the swapchain,
        // this is not an issue.
        let compatible_surface = window.map(|w| unsafe { instance.create_surface(w) });

        // Initialize the adapter (physical device).
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: compatible_surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::AdapterNone)?;

        // Create the `device` (and get the handle for the command queue `queue`)
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    limits: self.limits.clone(),
                    label: self.label,
                    features: self.features(&adapter),
                },
                self.trace_path,
            )
            .await
            .map_err(GpuError::RequestDeviceError)?;

        let preferred_format = compatible_surface.and_then(|s| s.get_preferred_format(&adapter));

        let profiler = Profiler::new(&device, &queue);

        let gpu = GpuCtx {
            instance,
            adapter,
            device,
            queue,
            profiler,
            preferred_format,
        };

        Ok(gpu.into_handle())
    }

    fn features(&self, adapter: &wgpu::Adapter) -> wgpu::Features {
        self.features | (self.optional_features & adapter.features())
    }
}

struct NoWindow;
unsafe impl HasRawWindowHandle for NoWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        unsafe { std::mem::zeroed() }
    }
}
