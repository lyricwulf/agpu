use core::num::NonZeroU32;

pub struct TextureBuilder<'a> {
    gpu: crate::GpuHandle,
    texture: wgpu::TextureDescriptor<'a>,
    view: wgpu::TextureViewDescriptor<'a>,
}

impl TextureBuilder<'_> {
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(gpu: crate::GpuHandle) -> Self {
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

    pub fn create<T>(mut self, data: &[T], size: &[u32]) -> crate::Texture
    where
        T: bytemuck::Pod,
    {
        let (width, height, depth_or_array_layers) = {
            let mut size = size.iter();
            (
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
            )
        };

        self.texture.size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        self.texture.dimension = size_dim(&self.texture.size);

        let texture = self.gpu.device.create_texture(&self.texture);
        let view = texture.create_view(&self.view);

        self.gpu.queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    self.texture.format.describe().block_size as u32 * width,
                ),
                rows_per_image: None,
            },
            self.texture.size,
        );

        crate::Texture {
            inner: texture,
            view,
            format: self.texture.format,
        }
    }

    /// Assumed that the input data is uniformly sized
    pub fn create2d<T>(mut self, data: &[&[T]]) -> crate::Texture
    where
        T: bytemuck::Pod,
    {
        let width = data.len() as u32;
        let height = data[0].len() as u32;
        self.texture.size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.texture.dimension = wgpu::TextureDimension::D2;

        let texture = self.gpu.device.create_texture(&self.texture);
        let view = texture.create_view(&self.view);
        self.gpu.queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(unsafe { flatten_2d(data) }),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(width),
                rows_per_image: None,
            },
            wgpu::Extent3d::default(),
        );

        crate::Texture {
            inner: texture,
            view,
            format: self.texture.format,
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

    pub const fn d1(mut self) -> Self {
        self.texture.dimension = wgpu::TextureDimension::D1;
        self
    }

    pub const fn d2(mut self) -> Self {
        self.texture.dimension = wgpu::TextureDimension::D2;
        self
    }

    pub const fn d3(mut self) -> Self {
        self.texture.dimension = wgpu::TextureDimension::D3;
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
}

impl crate::GpuHandle {
    pub fn new_texture<'a>(&self, label: &'a str) -> TextureBuilder<'a> {
        let mut builder = TextureBuilder::new(self.clone());
        builder.texture.label = Some(label);
        builder.view.label = Some(label);
        builder
    }
}

unsafe fn flatten_2d<'a, T>(data: &[&[T]]) -> &'a [T]
where
    T: bytemuck::Pod,
{
    std::slice::from_raw_parts(data.as_ptr() as *const T, data.len() * data[0].len())
}

const fn size_dim(size: &wgpu::Extent3d) -> wgpu::TextureDimension {
    if size.depth_or_array_layers == 1 {
        if size.height == 1 {
            wgpu::TextureDimension::D1
        } else {
            wgpu::TextureDimension::D2
        }
    } else {
        wgpu::TextureDimension::D3
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_dim() {
        assert_eq!(
            size_dim(&wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            }),
            wgpu::TextureDimension::D1
        );
        assert_eq!(
            size_dim(&wgpu::Extent3d {
                width: 10,
                height: 1,
                depth_or_array_layers: 1,
            }),
            wgpu::TextureDimension::D1
        );
        assert_eq!(
            size_dim(&wgpu::Extent3d {
                width: 10,
                height: 20,
                depth_or_array_layers: 1,
            }),
            wgpu::TextureDimension::D2
        );
        assert_eq!(
            size_dim(&wgpu::Extent3d {
                width: 20,
                height: 10,
                depth_or_array_layers: 30,
            }),
            wgpu::TextureDimension::D3
        );
    }
}
