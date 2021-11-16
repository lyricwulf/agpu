#[non_exhaustive]
#[derive(Debug)]
pub enum GpuError {
    AdapterNone,
    ShaderParseError,
    RequestDeviceError(wgpu::RequestDeviceError),
    DisplayNone,
    SurfaceError(wgpu::SurfaceError),
    BufferAsyncError,
    QueryNone,
}
impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for GpuError {}

/// Generic error type for any error.
/// Recommended to use with terminal errors only, which are expected to be displayed and not handled.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
