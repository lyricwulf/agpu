mod view;
pub use view::*;

mod sampler;
pub use sampler::*;

mod builder;
pub use builder::*;

use crate::GpuHandle;

// Re-export TextureFormat
pub use wgpu::TextureFormat;

pub struct Texture {
    pub(crate) gpu: GpuHandle,
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
    pub usage: wgpu::TextureUsages,
}
crate::wgpu_inner_deref!(Texture);
impl Texture {
    pub fn new(gpu: GpuHandle, desc: &wgpu::TextureDescriptor) -> Self {
        let inner = gpu.create_texture(desc);
        let view = inner.create_view(&Default::default());
        Self {
            gpu,
            inner,
            view,
            format: desc.format,
            size: desc.size,
            usage: desc.usage,
        }
    }

    pub fn resize(&mut self, size: &[u32]) {
        let (width, height, depth_or_array_layers) = {
            let mut size = size.iter();
            (
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
                *size.next().unwrap_or(&1),
            )
        };
        let new_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let new_usage = self.usage | wgpu::TextureUsages::COPY_DST;

        let new_texture = self.gpu.create_texture(&wgpu::TextureDescriptor {
            // TODO: update label for texture on resize
            label: None,
            size: new_size,
            // TODO: mip level count on resize??
            mip_level_count: 1,
            sample_count: 1,
            dimension: builder::size_dim(&self.size),
            format: self.format,
            usage: new_usage,
        });
        let mut enc = self.gpu.create_command_encoder("Texture resize encoder");
        enc.copy_texture_to_texture(
            self.inner.as_image_copy(),
            new_texture.as_image_copy(),
            self.size,
        );
        self.gpu.queue.submit([enc.finish()]);

        self.inner = new_texture;
        self.size = new_size;
        self.usage = new_usage;
    }

    pub fn write<T>(&self, data: &[T])
    where
        T: bytemuck::Pod,
    {
        self.write_at(0, data)
    }

    pub fn write_at<T>(&self, offset: wgpu::BufferAddress, data: &[T])
    where
        T: bytemuck::Pod,
    {
        let data_bytes = bytemuck::cast_slice::<_, u8>(data);
        self.gpu.queue.write_texture(
            self.inner.as_image_copy(),
            data_bytes,
            wgpu::ImageDataLayout {
                offset,
                bytes_per_row: std::num::NonZeroU32::new(data_bytes.len() as _),
                rows_per_image: std::num::NonZeroU32::new(1),
            },
            wgpu::Extent3d {
                width: data_bytes.len() as u32 / self.format.describe().block_size as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        )
    }

    pub fn write_at_texel<T>(&self, offset: wgpu::BufferAddress, data: &[T])
    where
        T: bytemuck::Pod,
    {
        self.write_at(
            offset * self.format.describe().block_size as wgpu::BufferAddress,
            data,
        )
    }

    // TODO
    #[allow(unreachable_code)]
    pub fn read_immediately(&self) -> Result<wgpu::util::DownloadBuffer, wgpu::BufferAsyncError> {
        todo!("Texture::read_immediately :(");
        let texel_count = self.size.width * self.size.height * self.size.depth_or_array_layers;
        let size = texel_count * self.format.describe().block_size as u32;
        let staging_buf = self
            .gpu
            .new_buffer("texture read staging buffer")
            .allow_copy_to()
            .allow_map_read()
            .create_uninit(size as _);
        let mut enc = self
            .gpu
            .create_command_encoder("texture read immediately enc");

        let staging_copy = wgpu::ImageCopyBuffer {
            buffer: &staging_buf,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(size),
                rows_per_image: std::num::NonZeroU32::new(256),
            },
        };

        enc.copy_texture_to_buffer(self.inner.as_image_copy(), staging_copy, self.size);

        self.gpu.queue.submit([enc.finish()]);

        staging_buf.download_immediately()
    }
}

// TODO
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn texture_write() {
//         let data = [10_u32, 20, 30];

//         let gpu = crate::Gpu::builder().build_headless().unwrap();
//         let texture = gpu
//             .new_texture("resize test")
//             .allow_copy_from()
//             .create_empty(&[128, 128]);
//         texture.write(&data);

//         let texture_read = texture.read_immediately().unwrap();
//         let texture_read = bytemuck::cast_slice::<_, u32>(&texture_read);

//         assert_eq!(data, texture_read[..data.len()]);
//     }
// }
