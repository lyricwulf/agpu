mod view;
pub use view::*;

mod sampler;
pub use sampler::*;

mod builder;
pub use builder::*;

use crate::GpuHandle;

// Re-export TextureFormat
pub use wgpu::TextureFormat;

pub struct Texture<D>
where
    D: TextureDimensions,
{
    pub(crate) gpu: GpuHandle,
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: D,
    pub usage: wgpu::TextureUsages,
}
impl<D> std::ops::Deref for Texture<D>
where
    D: TextureDimensions,
{
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<D> std::ops::DerefMut for Texture<D>
where
    D: TextureDimensions,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<D> Texture<D>
where
    D: crate::TextureDimensions,
{
    // FIXME
    // pub fn new(gpu: GpuHandle, desc: &wgpu::TextureDescriptor) -> Self {
    //     let inner = gpu.create_texture(desc);
    //     let view = inner.create_view(&Default::default());
    //     Self {
    //         gpu,
    //         inner,
    //         view,
    //         format: desc.format,
    //         size: desc.size,
    //         usage: desc.usage,
    //     }
    // }

    pub fn resize(&mut self, size: D) {
        let new_usage = self.usage | wgpu::TextureUsages::COPY_DST;

        let new_texture = self.gpu.create_texture(&wgpu::TextureDescriptor {
            // TODO: update label for texture on resize
            label: None,
            size: size.as_extent(),
            // TODO: mip level count on resize??
            mip_level_count: 1,
            sample_count: 1,
            dimension: size.dim(),
            format: self.format,
            usage: new_usage,
        });

        let mut enc = self.gpu.create_command_encoder("Texture resize encoder");
        enc.copy_texture_to_texture(
            self.inner.as_image_copy(),
            new_texture.as_image_copy(),
            self.size.as_extent(),
        );
        self.gpu.queue.submit([enc.finish()]);

        self.inner = new_texture;
        self.size = size;
        self.usage = new_usage;
    }

    pub fn write<T>(&self, size: D, data: &[T])
    where
        T: bytemuck::Pod,
    {
        self.write_block(D::ZEROED, size, data)
    }

    pub fn write_block<T>(&self, texel: D, size: D, data: &[T])
    where
        T: bytemuck::Pod,
    {
        let data_bytes = bytemuck::cast_slice::<_, u8>(data);
        self.gpu.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &self.inner,
                mip_level: 0,
                origin: texel.as_origin(),
                aspect: wgpu::TextureAspect::All,
            },
            data_bytes,
            wgpu::ImageDataLayout {
                // This is 0 because our source should not be offset
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    self.size.width() * self.format.describe().block_size as u32,
                ),
                rows_per_image: None,
            },
            size.as_extent(),
        )
    }

    // TODO
    #[allow(unreachable_code)]
    pub fn read_immediately(&self) -> Result<wgpu::util::DownloadBuffer, wgpu::BufferAsyncError> {
        let format = self.format.describe();
        let texel_count = self.size.width() * self.size.height() * self.size.depth();
        let read_size = texel_count * format.block_size as u32
            / (format.block_dimensions.0 as u32 * format.block_dimensions.1 as u32);
        println!("texel_count: {}, read_size: {}", texel_count, read_size);
        let staging_buf = self
            .gpu
            .new_buffer("texture read staging buffer")
            .allow_copy_to()
            .allow_map_read()
            .create_uninit(read_size as _);

        let padded_bytes_per_row = {
            let bytes_per_pixel = format.block_size as u32;
            let unpadded_bytes_per_row = self.size.width() * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
            let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
            padded_bytes_per_row
        };

        let staging_copy = wgpu::ImageCopyBuffer {
            buffer: &staging_buf,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row),
                rows_per_image: None,
            },
        };

        let mut enc = self
            .gpu
            .create_command_encoder("texture read immediately enc");

        enc.copy_texture_to_buffer(
            self.inner.as_image_copy(),
            staging_copy,
            self.size.as_extent(),
        );

        self.gpu.queue.submit([enc.finish()]);

        staging_buf.download_immediately()
    }
}

pub type D1 = (u32,);
pub type D2 = (u32, u32);
pub type D3 = (u32, u32, u32);

pub trait TextureDimensions: Copy {
    const ZEROED: Self;
    fn dim(&self) -> wgpu::TextureDimension;
    fn as_extent(&self) -> wgpu::Extent3d;
    fn as_origin(&self) -> wgpu::Origin3d;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
}

impl TextureDimensions for (u32, u32, u32) {
    const ZEROED: Self = (0, 0, 0);
    fn dim(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D3
    }
    fn as_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: self.2,
        }
    }
    fn as_origin(&self) -> wgpu::Origin3d {
        wgpu::Origin3d {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
    fn width(&self) -> u32 {
        self.0
    }
    fn height(&self) -> u32 {
        self.1
    }
    fn depth(&self) -> u32 {
        self.2
    }
}

impl TextureDimensions for (u32, u32) {
    const ZEROED: Self = (0, 0);
    fn dim(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D2
    }
    fn as_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: 1,
        }
    }
    fn as_origin(&self) -> wgpu::Origin3d {
        wgpu::Origin3d {
            x: self.0,
            y: self.1,
            z: 0,
        }
    }
    fn width(&self) -> u32 {
        self.0
    }
    fn height(&self) -> u32 {
        self.1
    }
    fn depth(&self) -> u32 {
        1
    }
}

impl TextureDimensions for (u32,) {
    const ZEROED: Self = (0,);
    fn dim(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D1
    }
    fn as_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.0,
            height: 1,
            depth_or_array_layers: 1,
        }
    }
    fn as_origin(&self) -> wgpu::Origin3d {
        wgpu::Origin3d {
            x: self.0,
            y: 0,
            z: 0,
        }
    }
    fn width(&self) -> u32 {
        self.0
    }
    fn height(&self) -> u32 {
        1
    }
    fn depth(&self) -> u32 {
        1
    }
}

// TODO
#[cfg(test)]
mod tests {
    #[test]
    fn texture_write() {
        let data = [10_u32; 99999];

        let gpu = crate::Gpu::builder().build_headless().unwrap();
        let texture = gpu
            .new_texture("resize test")
            .allow_copy_from()
            .create_empty((1024, 1024));
        texture.write((256, 23), &data);

        let texture_read = texture.read_immediately().unwrap();
        let texture_read = bytemuck::cast_slice::<_, u32>(&texture_read);

        assert_eq!(data, texture_read[..data.len()]);
    }
}
