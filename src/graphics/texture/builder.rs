use core::num::NonZeroU32;

pub struct TextureBuilder<'a> {
    gpu: crate::Gpu,
    texture: wgpu::TextureDescriptor<'a>,
    view: wgpu::TextureViewDescriptor<'a>,
}

impl TextureBuilder<'_> {
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(gpu: crate::Gpu) -> Self {
        Self {
            gpu,
            texture: wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d::default(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEFAULT_FORMAT,
                usage: wgpu::TextureUsages::empty(),
            },
            view: Default::default(),
        }
    }

    /// Create the buffer with the given data as its contents.
    /// Implicitly adds the `COPY_DST` usage if it is not present in the descriptor,
    /// as it is required to be able to upload the data to the gpu.
    pub fn create<T, D>(mut self, size: D, data: &[T]) -> crate::Texture<D>
    where
        T: bytemuck::Pod,
        D: crate::TextureDimensions,
    {
        self.texture.usage |= wgpu::TextureUsages::COPY_DST;
        self.texture.size = size.as_extent();
        self.texture.dimension = size.dim();

        let texture = self.gpu.device.create_texture(&self.texture);
        let view = texture.create_view(&self.view);

        self.gpu.queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    self.texture.format.describe().block_size as u32 * size.width(),
                ),
                rows_per_image: NonZeroU32::new(size.height()),
            },
            self.texture.size,
        );

        crate::Texture {
            gpu: self.gpu,
            inner: texture,
            view,
            format: self.texture.format,
            size,
            usage: self.texture.usage,
        }
    }

    /// Creates the texture with a given size, and filled with 0.
    pub fn create_empty<D>(mut self, size: D) -> crate::Texture<D>
    where
        D: crate::TextureDimensions,
    {
        self.texture.usage |= wgpu::TextureUsages::COPY_DST;
        self.texture.size = size.as_extent();
        self.texture.dimension = size.dim();

        let texture = self.gpu.device.create_texture(&self.texture);
        let view = texture.create_view(&self.view);

        crate::Texture {
            gpu: self.gpu,
            inner: texture,
            view,
            format: self.texture.format,
            size,
            usage: self.texture.usage,
        }
    }

    pub const fn mips(mut self, mips: u32) -> Self {
        self.texture.mip_level_count = mips;
        self
    }

    pub const fn multisample(mut self, samples: u32) -> Self {
        self.texture.sample_count = samples;
        self
    }

    pub const fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.texture.format = format;
        self
    }

    pub fn allow_copy_from(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::COPY_SRC;
        self
    }

    pub fn allow_copy_to(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::COPY_DST;
        self
    }

    pub fn allow_binding(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        self
    }

    pub fn allow_storage_binding(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::STORAGE_BINDING;
        self
    }

    pub fn as_render_target(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self
    }

    pub fn as_depth(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self.texture.format = wgpu::TextureFormat::Depth32Float;
        self
    }

    pub fn as_depth24(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self.texture.format = wgpu::TextureFormat::Depth24Plus;
        self
    }

    pub fn as_depth_stencil(mut self) -> Self {
        self.texture.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self.texture.format = wgpu::TextureFormat::Depth24PlusStencil8;
        self
    }
}

impl crate::Gpu {
    pub fn new_texture<'a>(&self, label: &'a str) -> TextureBuilder<'a> {
        let mut builder = TextureBuilder::new(self.clone());
        builder.texture.label = Some(label);
        builder.view.label = Some(label);
        builder
    }
}
