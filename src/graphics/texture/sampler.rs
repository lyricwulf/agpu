use crate::Binding;

// 'a: Label str
pub struct SamplerBuilder<'a, 'gpu> {
    pub gpu: &'gpu wgpu::Device,
    pub inner: wgpu::SamplerDescriptor<'a>,
}

impl SamplerBuilder<'_, '_> {
    pub fn create(&self) -> Sampler {
        let inner = self.gpu.create_sampler(&self.inner);
        let filtering = self.inner.mag_filter == wgpu::FilterMode::Linear
            || self.inner.min_filter == wgpu::FilterMode::Linear
            || self.inner.mipmap_filter == wgpu::FilterMode::Linear;
        let comparison = self.inner.compare.is_some();
        Sampler {
            inner,
            filtering,
            comparison,
        }
    }

    /// How to deal with out of bounds accesses
    /// xyz/uvw are all affected (for now)
    pub const fn tile(mut self) -> Self {
        self.inner.address_mode_u = wgpu::AddressMode::Repeat;
        self.inner.address_mode_v = wgpu::AddressMode::Repeat;
        self.inner.address_mode_w = wgpu::AddressMode::Repeat;
        self
    }

    pub const fn mirror_tile(mut self) -> Self {
        self.inner.address_mode_u = wgpu::AddressMode::MirrorRepeat;
        self.inner.address_mode_v = wgpu::AddressMode::MirrorRepeat;
        self.inner.address_mode_w = wgpu::AddressMode::MirrorRepeat;
        self
    }

    const fn clamp_to_border(mut self) -> Self {
        self.inner.address_mode_u = wgpu::AddressMode::ClampToBorder;
        self.inner.address_mode_v = wgpu::AddressMode::ClampToBorder;
        self.inner.address_mode_w = wgpu::AddressMode::ClampToBorder;
        self
    }

    pub const fn oob_transparent(mut self) -> Self {
        self.inner.border_color = Some(wgpu::SamplerBorderColor::TransparentBlack);
        self.clamp_to_border()
    }

    pub const fn oob_black(mut self) -> Self {
        self.inner.border_color = Some(wgpu::SamplerBorderColor::OpaqueBlack);
        self.clamp_to_border()
    }

    pub const fn oob_white(mut self) -> Self {
        self.inner.border_color = Some(wgpu::SamplerBorderColor::OpaqueWhite);
        self.clamp_to_border()
    }

    pub const fn linear_filter(mut self) -> Self {
        self.inner.mag_filter = wgpu::FilterMode::Linear;
        self.inner.min_filter = wgpu::FilterMode::Linear;
        self.inner.mipmap_filter = wgpu::FilterMode::Linear;
        self
    }

    pub const fn lod_range(mut self, range: std::ops::Range<f32>) -> Self {
        self.inner.lod_min_clamp = range.start;
        self.inner.lod_max_clamp = range.end;
        self
    }

    pub const fn comparator(mut self, comparator: wgpu::CompareFunction) -> Self {
        self.inner.compare = Some(comparator);
        self
    }
}
impl crate::Gpu {
    // Named new_sampler so not to shadow create_sampler
    pub fn new_sampler<'a, 'gpu>(&'gpu self, label: &'a str) -> SamplerBuilder<'a, 'gpu> {
        SamplerBuilder {
            gpu: &self.device,
            inner: wgpu::SamplerDescriptor {
                label: Some(label),
                ..Default::default()
            },
        }
    }
}

pub struct Sampler {
    pub inner: wgpu::Sampler,
    filtering: bool,
    comparison: bool,
}
crate::wgpu_inner_deref!(Sampler);
impl Sampler {
    pub fn bind(&self) -> Binding {
        // TODO: Decide if samplers type should be automatically determined,
        // just because the sampler CAN be used as comparison or filtering,
        // maybe doesn't necessarily mean that we will bind it as such
        let ty = match (self.comparison, self.filtering) {
            (true, _) => wgpu::SamplerBindingType::Comparison,
            (false, true) => wgpu::SamplerBindingType::Filtering,
            (false, false) => wgpu::SamplerBindingType::NonFiltering,
        };

        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Sampler(ty),
            resource: wgpu::BindingResource::Sampler(self),
        }
    }
}
