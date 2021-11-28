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
    pub fn bind_ubo(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: core::num::NonZeroU64::new(self.size),
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
    pub fn bind_ssbo(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: core::num::NonZeroU64::new(self.size),
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
    pub fn bind_ssbo_readonly(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: core::num::NonZeroU64::new(self.size),
            },
            resource: self.as_entire_binding(),
        }
    }
}

impl crate::Texture {
    /// Create a textureview binding.
    pub fn bind_texture(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }

    // Create a storage texture binding.
    pub fn bind_storage_texture(&self) -> Binding {
        Binding {
            visibility: Binding::DEFAULT_VISIBILITY,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
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

#[derive(Clone)]
pub struct Binding<'a> {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub resource: wgpu::BindingResource<'a>,
}
impl Binding<'_> {
    pub(crate) const DEFAULT_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::all();

    gen_binding_vis_fn!(
        in_none => NONE,
        in_vertex => VERTEX,
        in_fragment => FRAGMENT,
        in_compute => COMPUTE,
        in_vertex_fragment => VERTEX_FRAGMENT
    );

    pub fn buffer_dynamic_offset(mut self) -> Self {
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
            panic!("dynamic_offset is only supported for uniform buffers");
        }
        self
    }
}

pub struct BindingGroup {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl BindingGroup {
    pub fn new(device: &wgpu::Device, bindings: &[Binding]) -> Self {
        bindings[0].clone().in_vertex();
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

        BindingGroup {
            bind_group_layout,
            bind_group,
        }
    }
}
impl crate::Gpu {
    pub fn create_binding_group(&self, bindings: &[Binding]) -> BindingGroup {
        BindingGroup::new(&self.device, bindings)
    }
}

// // Desired usage:
// // [sampler.binding(), buffer.binding(), texture.binding()];

// pub struct BindingType {
//     inner: wgpu::BindingType,
// }
// impl std::ops::Deref for BindingType {
//     type Target = wgpu::BindingType;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }
// impl std::ops::DerefMut for BindingType {
//     fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
//         &mut self.inner
//     }
// }
// impl BindingType {}
