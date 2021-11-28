use std::mem::ManuallyDrop;
use std::ops::Deref;

/// Utility struct for creating a `BufferView` that auto unmaps when dropped.
pub struct ScopedBufferView<'a> {
    /// The inner buffer view.
    /// This is `Option` because it needs to be dropped before the buffer can be
    /// unmapped, however it will not be `None` until the scope is dropped.
    buffer_view: ManuallyDrop<wgpu::BufferView<'a>>,
    /// Reference to the Buffer so we can unmap it when the view is dropped.
    buffer: &'a wgpu::Buffer,
}
impl<'a> ScopedBufferView<'a> {
    pub fn new(
        buffer: &'a wgpu::Buffer,
        buffer_view: wgpu::BufferView<'a>,
    ) -> ScopedBufferView<'a> {
        // Wrap in ManuallyDrop
        let buffer_view = ManuallyDrop::new(buffer_view);
        ScopedBufferView {
            buffer_view,
            buffer,
        }
    }
}
impl<'a> Deref for ScopedBufferView<'a> {
    type Target = wgpu::BufferView<'a>;
    fn deref(&self) -> &Self::Target {
        &self.buffer_view
    }
}
impl Drop for ScopedBufferView<'_> {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.buffer_view) }
        self.buffer.unmap();
    }
}
