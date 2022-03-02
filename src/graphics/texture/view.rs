use std::ops::{Deref, DerefMut};

use crate::Gpu;

pub struct TextureView<'a> {
    pub gpu: &'a Gpu,
    pub inner: wgpu::TextureView,
}
impl Deref for TextureView<'_> {
    type Target = wgpu::TextureView;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for TextureView<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> TextureView<'a> {
    /// Bind the texture view to the given bind group.
    /// This assumes the texture view is a filterable float format with dimension 2
    pub fn bind(&self) -> crate::Binding<'_> {
        crate::Binding {
            gpu: self.gpu,
            visibility: crate::Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            resource: wgpu::BindingResource::TextureView(self),
        }
    }

    pub fn attach_render(&self) -> crate::RenderAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: self,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    }

    /// Since traits are not const we have this to Deref const
    pub const fn deref_const(&self) -> &wgpu::TextureView {
        &self.inner
    }
}

// impl<'a> From<wgpu::TextureView> for TextureView<'a> {
//     fn from(inner: wgpu::TextureView) -> Self {
//         TextureView { inner }
//     }
// }
