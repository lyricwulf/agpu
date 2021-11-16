use std::ops::Deref;

/// Utility struct for creating a `BufferView` that auto unmaps when dropped.
pub struct ScopedBufferView<'a> {
    /// The inner buffer view.
    /// This is `Option` because it needs to be dropped before the buffer can be
    /// unmapped, however it will not be `None` until the scope is dropped.
    buffer_view: Option<wgpu::BufferView<'a>>,
    /// Reference to the Buffer so we can unmap it when the view is dropped.
    buffer: &'a wgpu::Buffer,
}
impl<'a> ScopedBufferView<'a> {
    pub fn new(
        buffer: &'a wgpu::Buffer,
        buffer_view: wgpu::BufferView<'a>,
    ) -> ScopedBufferView<'a> {
        // Wrap in Option
        let buffer_view = Some(buffer_view);
        ScopedBufferView {
            buffer_view,
            buffer,
        }
    }
}
impl<'a> Deref for ScopedBufferView<'a> {
    type Target = wgpu::BufferView<'a>;
    fn deref(&self) -> &Self::Target {
        // We can always unwrap this
        self.buffer_view.as_ref().unwrap()
    }
}
impl Drop for ScopedBufferView<'_> {
    fn drop(&mut self) {
        self.buffer_view = None;
        self.buffer.unmap();
    }
}
