use std::ops::Deref;

// Re-export TextureFormat
pub use wgpu::TextureFormat;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TexFormat(pub TextureFormat);
pub trait TextureFormatExt {
    fn to_agpu(self) -> TexFormat;
}
impl TextureFormatExt for TextureFormat {
    fn to_agpu(self) -> TexFormat {
        self.into()
    }
}

impl Deref for TexFormat {
    type Target = TextureFormat;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TexFormat {
    pub const fn new(format: wgpu::TextureFormat) -> Self {
        Self(format)
    }

    pub const fn to_wgpu(self) -> wgpu::TextureFormat {
        self.0
    }

    pub const fn target(self) -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format: self.to_wgpu(),
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }
    }
}

impl From<wgpu::TextureFormat> for TexFormat {
    fn from(format: wgpu::TextureFormat) -> Self {
        TexFormat(format)
    }
}

impl From<TexFormat> for wgpu::TextureFormat {
    fn from(format: TexFormat) -> Self {
        format.0
    }
}

impl TexFormat {
    /// Return the linear (non-SRGB) color variant of the format
    pub const fn linear(self) -> Self {
        TexFormat(match self.0 {
            TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8Unorm,
            TextureFormat::Bc1RgbaUnormSrgb => TextureFormat::Bc1RgbaUnorm,
            TextureFormat::Bc2RgbaUnormSrgb => TextureFormat::Bc2RgbaUnorm,
            TextureFormat::Bc3RgbaUnormSrgb => TextureFormat::Bc3RgbaUnorm,
            TextureFormat::Bc7RgbaUnormSrgb => TextureFormat::Bc7RgbaUnorm,
            TextureFormat::Etc2Rgb8UnormSrgb => TextureFormat::Etc2Rgb8Unorm,
            TextureFormat::Etc2Rgb8A1UnormSrgb => TextureFormat::Etc2Rgb8A1Unorm,
            TextureFormat::Etc2Rgba8UnormSrgb => TextureFormat::Etc2Rgba8Unorm,
            TextureFormat::Astc4x4RgbaUnormSrgb => TextureFormat::Astc4x4RgbaUnorm,
            TextureFormat::Astc5x4RgbaUnormSrgb => TextureFormat::Astc5x4RgbaUnorm,
            TextureFormat::Astc5x5RgbaUnormSrgb => TextureFormat::Astc5x5RgbaUnorm,
            TextureFormat::Astc6x5RgbaUnormSrgb => TextureFormat::Astc6x5RgbaUnorm,
            TextureFormat::Astc6x6RgbaUnormSrgb => TextureFormat::Astc6x6RgbaUnorm,
            TextureFormat::Astc8x5RgbaUnormSrgb => TextureFormat::Astc8x5RgbaUnorm,
            TextureFormat::Astc8x6RgbaUnormSrgb => TextureFormat::Astc8x6RgbaUnorm,
            _ => return self,
        })
    }

    /// Return the SRGB variant of the format
    pub const fn srgb(self) -> Self {
        TexFormat(match self.0 {
            TextureFormat::Rgba8Unorm => TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm => TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Bc1RgbaUnorm => TextureFormat::Bc1RgbaUnormSrgb,
            TextureFormat::Bc2RgbaUnorm => TextureFormat::Bc2RgbaUnormSrgb,
            TextureFormat::Bc3RgbaUnorm => TextureFormat::Bc3RgbaUnormSrgb,
            TextureFormat::Bc7RgbaUnorm => TextureFormat::Bc7RgbaUnormSrgb,
            TextureFormat::Etc2Rgb8Unorm => TextureFormat::Etc2Rgb8UnormSrgb,
            TextureFormat::Etc2Rgb8A1Unorm => TextureFormat::Etc2Rgb8A1UnormSrgb,
            TextureFormat::Etc2Rgba8Unorm => TextureFormat::Etc2Rgba8UnormSrgb,
            TextureFormat::Astc4x4RgbaUnorm => TextureFormat::Astc4x4RgbaUnormSrgb,
            TextureFormat::Astc5x4RgbaUnorm => TextureFormat::Astc5x4RgbaUnormSrgb,
            TextureFormat::Astc5x5RgbaUnorm => TextureFormat::Astc5x5RgbaUnormSrgb,
            TextureFormat::Astc6x5RgbaUnorm => TextureFormat::Astc6x5RgbaUnormSrgb,
            TextureFormat::Astc6x6RgbaUnorm => TextureFormat::Astc6x6RgbaUnormSrgb,
            TextureFormat::Astc8x5RgbaUnorm => TextureFormat::Astc8x5RgbaUnormSrgb,
            TextureFormat::Astc8x6RgbaUnorm => TextureFormat::Astc8x6RgbaUnormSrgb,
            _ => return self,
        })
    }
}
