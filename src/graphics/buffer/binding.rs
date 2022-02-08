use wgpu::BindGroupEntry;

impl crate::Buffer {
    /// Create a uniform buffer binding.
    ///
    /// Example GLSL syntax:
    /// ```cpp,ignore
    /// layout(std140, binding = 0)
    /// uniform Globals {
    ///     vec2 aUniform;
    ///     vec2 anotherUniform;
    /// };
    /// ```
    #[must_use]
    pub fn bind_uniform(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            resource: self.as_entire_binding(),
        }
    }

    /// Create a storage buffer binding.
    ///
    /// Example GLSL syntax:
    /// ```cpp,ignore
    /// layout (set=0, binding=0) buffer myStorageBuffer {
    ///     vec4 myElement[];
    /// };
    /// ```
    #[must_use]
    pub fn bind_storage(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            resource: self.as_entire_binding(),
        }
    }

    /// Create a storage buffer binding. The buffer is read-only in shader,
    /// and it must be annotated with `readonly`.
    ///
    /// Example GLSL syntax:
    /// ```cpp,ignore
    /// layout (set=0, binding=0) readonly buffer myStorageBuffer {
    ///     vec4 myElement[];
    /// };
    /// ```
    #[must_use]
    pub fn bind_storage_readonly(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            resource: self.as_entire_binding(),
        }
    }
}

impl<D> crate::Texture<D>
where
    D: crate::TextureDimensions,
{
    /// Create a textureview binding.
    // Can be const following RFC 2632
    pub fn bind_texture(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Texture {
                sample_type: sample_type(self.format),
                // TODO: different texture view reps?
                view_dimension: match self.size.dim() {
                    wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
                    wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
                    wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
                },
                multisampled: false,
            },
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }

    /// Alias for `bind_texture()`.
    // Can be const following RFC 2632
    pub fn bind(&self) -> Binding {
        self.bind_texture()
    }

    /// Create a storage texture binding.
    // Can be const following RFC 2632
    pub fn bind_storage_texture(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::StorageTexture {
                // TODO: different texture view reps?
                view_dimension: match self.size.dim() {
                    wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
                    wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
                    wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
                },
                access: wgpu::StorageTextureAccess::ReadWrite,
                format: self.format,
            },
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

macro_rules! gen_binding_vis_fn {
    ($($fn_name:ident => $stage:ident),*) => {
        $(
            pub const fn $fn_name(mut self) -> Self {
                self.visibility = ::wgpu::ShaderStages::$stage;
                self
            }
        )*
    };
}

#[derive(Clone, Debug)]
pub struct Binding<'a> {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub resource: wgpu::BindingResource<'a>,
}
impl Binding<'_> {
    pub(crate) const DEFAULT_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;

    gen_binding_vis_fn!(
        in_none => NONE,
        in_vertex => VERTEX,
        in_fragment => FRAGMENT,
        in_compute => COMPUTE,
        in_vertex_fragment => VERTEX_FRAGMENT
    );

    pub const fn buffer_dynamic_offset(mut self) -> Self {
        if let wgpu::BindingType::Buffer {
            ty,
            min_binding_size,
            ..
        } = self.ty
        {
            self.ty = wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: true,
                min_binding_size,
            };
        } else {
            #[cfg(feature = "const_panic")]
            panic!("dynamic_offset is only supported for uniform buffers");
        }
        self
    }

    pub const fn sample_uint(mut self) -> Self {
        if let wgpu::BindingType::Texture {
            view_dimension,
            multisampled,
            ..
        } = self.ty
        {
            self.ty = wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Uint,
                view_dimension,
                multisampled,
            };
        } else {
            #[cfg(feature = "const_panic")]
            panic!("sample_uint is only supported for textures");
        }
        self
    }
}

#[derive(Debug)]
pub struct BindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub inner: wgpu::BindGroup,
}
crate::wgpu_inner_deref!(BindGroup);

impl BindGroup {
    pub fn new(device: &wgpu::Device, bindings: &[Binding]) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: bindings
                .iter()
                .enumerate()
                .map(|(i, binding)| wgpu::BindGroupLayoutEntry {
                    binding: i as _,
                    visibility: binding.visibility,
                    ty: binding.ty,
                    count: None,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: bindings
                .iter()
                .enumerate()
                .map(|(i, b)| BindGroupEntry {
                    binding: i as _,
                    resource: b.resource.clone(),
                })
                .collect::<Vec<_>>()
                .as_slice(),
        });

        BindGroup {
            layout: bind_group_layout,
            inner: bind_group,
        }
    }
}
impl crate::Gpu {
    pub fn create_bind_group(&self, bindings: &[Binding]) -> BindGroup {
        BindGroup::new(&self.device, bindings)
    }
}

/// Returns the binding sample type for a texture format.
pub(crate) const fn sample_type(format: wgpu::TextureFormat) -> wgpu::TextureSampleType {
    // Sample Types
    const UINT: wgpu::TextureSampleType = wgpu::TextureSampleType::Uint;
    const SINT: wgpu::TextureSampleType = wgpu::TextureSampleType::Sint;
    const NEAREST: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    const FLOAT: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    const DEPTH: wgpu::TextureSampleType = wgpu::TextureSampleType::Depth;

    match format {
        // Normal 8 bit textures
        wgpu::TextureFormat::R8Unorm => FLOAT,
        wgpu::TextureFormat::R8Snorm => FLOAT,
        wgpu::TextureFormat::R8Uint => UINT,
        wgpu::TextureFormat::R8Sint => SINT,

        // Normal 16 bit textures
        wgpu::TextureFormat::R16Uint => UINT,
        wgpu::TextureFormat::R16Sint => SINT,
        wgpu::TextureFormat::R16Float => FLOAT,
        wgpu::TextureFormat::Rg8Unorm => FLOAT,
        wgpu::TextureFormat::Rg8Snorm => FLOAT,
        wgpu::TextureFormat::Rg8Uint => UINT,
        wgpu::TextureFormat::Rg8Sint => SINT,

        // Normal 32 bit textures
        wgpu::TextureFormat::R32Uint => UINT,
        wgpu::TextureFormat::R32Sint => SINT,
        wgpu::TextureFormat::R32Float => NEAREST,
        wgpu::TextureFormat::Rg16Uint => UINT,
        wgpu::TextureFormat::Rg16Sint => SINT,
        wgpu::TextureFormat::Rg16Float => FLOAT,
        wgpu::TextureFormat::Rgba8Unorm => FLOAT,
        wgpu::TextureFormat::Rgba8UnormSrgb => FLOAT,
        wgpu::TextureFormat::Rgba8Snorm => FLOAT,
        wgpu::TextureFormat::Rgba8Uint => UINT,
        wgpu::TextureFormat::Rgba8Sint => SINT,
        wgpu::TextureFormat::Bgra8Unorm => FLOAT,
        wgpu::TextureFormat::Bgra8UnormSrgb => FLOAT,

        // Packed 32 bit textures
        wgpu::TextureFormat::Rgb10a2Unorm => FLOAT,
        wgpu::TextureFormat::Rg11b10Float => FLOAT,

        // Packed 32 bit textures
        wgpu::TextureFormat::Rg32Uint => UINT,
        wgpu::TextureFormat::Rg32Sint => SINT,
        wgpu::TextureFormat::Rg32Float => NEAREST,
        wgpu::TextureFormat::Rgba16Uint => UINT,
        wgpu::TextureFormat::Rgba16Sint => SINT,
        wgpu::TextureFormat::Rgba16Float => FLOAT,

        // Packed 32 bit textures
        wgpu::TextureFormat::Rgba32Uint => UINT,
        wgpu::TextureFormat::Rgba32Sint => SINT,
        wgpu::TextureFormat::Rgba32Float => NEAREST,

        // Depth-stencil textures
        wgpu::TextureFormat::Depth32Float => DEPTH,
        wgpu::TextureFormat::Depth24Plus => DEPTH,
        wgpu::TextureFormat::Depth24PlusStencil8 => DEPTH,

        // Packed uncompressed
        wgpu::TextureFormat::Rgb9e5Ufloat => FLOAT,

        // BCn compressed textures
        wgpu::TextureFormat::Bc1RgbaUnorm
        | wgpu::TextureFormat::Bc1RgbaUnormSrgb
        | wgpu::TextureFormat::Bc2RgbaUnorm
        | wgpu::TextureFormat::Bc2RgbaUnormSrgb
        | wgpu::TextureFormat::Bc3RgbaUnorm
        | wgpu::TextureFormat::Bc3RgbaUnormSrgb
        | wgpu::TextureFormat::Bc4RUnorm
        | wgpu::TextureFormat::Bc4RSnorm
        | wgpu::TextureFormat::Bc5RgUnorm
        | wgpu::TextureFormat::Bc5RgSnorm
        | wgpu::TextureFormat::Bc6hRgbUfloat
        | wgpu::TextureFormat::Bc6hRgbSfloat
        | wgpu::TextureFormat::Bc7RgbaUnorm
        | wgpu::TextureFormat::Bc7RgbaUnormSrgb => FLOAT,

        // ETC compressed textures
        wgpu::TextureFormat::Etc2Rgb8Unorm
        | wgpu::TextureFormat::Etc2Rgb8UnormSrgb
        | wgpu::TextureFormat::Etc2Rgb8A1Unorm
        | wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb
        | wgpu::TextureFormat::Etc2Rgba8Unorm
        | wgpu::TextureFormat::Etc2Rgba8UnormSrgb
        | wgpu::TextureFormat::EacR11Unorm
        | wgpu::TextureFormat::EacR11Snorm
        | wgpu::TextureFormat::EacRg11Unorm
        | wgpu::TextureFormat::EacRg11Snorm => FLOAT,

        // ASTC compressed textures
        wgpu::TextureFormat::Astc4x4RgbaUnorm
        | wgpu::TextureFormat::Astc4x4RgbaUnormSrgb
        | wgpu::TextureFormat::Astc5x4RgbaUnorm
        | wgpu::TextureFormat::Astc5x4RgbaUnormSrgb
        | wgpu::TextureFormat::Astc5x5RgbaUnorm
        | wgpu::TextureFormat::Astc5x5RgbaUnormSrgb
        | wgpu::TextureFormat::Astc6x5RgbaUnorm
        | wgpu::TextureFormat::Astc6x5RgbaUnormSrgb
        | wgpu::TextureFormat::Astc6x6RgbaUnorm
        | wgpu::TextureFormat::Astc6x6RgbaUnormSrgb
        | wgpu::TextureFormat::Astc8x5RgbaUnorm
        | wgpu::TextureFormat::Astc8x5RgbaUnormSrgb
        | wgpu::TextureFormat::Astc8x6RgbaUnorm
        | wgpu::TextureFormat::Astc8x6RgbaUnormSrgb
        | wgpu::TextureFormat::Astc10x5RgbaUnorm
        | wgpu::TextureFormat::Astc10x5RgbaUnormSrgb
        | wgpu::TextureFormat::Astc10x6RgbaUnorm
        | wgpu::TextureFormat::Astc10x6RgbaUnormSrgb
        | wgpu::TextureFormat::Astc8x8RgbaUnorm
        | wgpu::TextureFormat::Astc8x8RgbaUnormSrgb
        | wgpu::TextureFormat::Astc10x8RgbaUnorm
        | wgpu::TextureFormat::Astc10x8RgbaUnormSrgb
        | wgpu::TextureFormat::Astc10x10RgbaUnorm
        | wgpu::TextureFormat::Astc10x10RgbaUnormSrgb
        | wgpu::TextureFormat::Astc12x10RgbaUnorm
        | wgpu::TextureFormat::Astc12x10RgbaUnormSrgb
        | wgpu::TextureFormat::Astc12x12RgbaUnorm
        | wgpu::TextureFormat::Astc12x12RgbaUnormSrgb => FLOAT,

        // Optional normalized 16-bit-per-channel formats
        wgpu::TextureFormat::R16Unorm
        | wgpu::TextureFormat::R16Snorm
        | wgpu::TextureFormat::Rg16Unorm
        | wgpu::TextureFormat::Rg16Snorm
        | wgpu::TextureFormat::Rgba16Unorm
        | wgpu::TextureFormat::Rgba16Snorm => FLOAT,
    }
}
