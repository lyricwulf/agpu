use std::borrow::Cow;

use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;

use crate::GpuError;
use crate::GpuHandle;

use crate::RenderPipeline;

pub struct PipelineBuilder<'a> {
    /// Handle to the Gpu
    gpu: GpuHandle,
    label: Option<&'a str>,
    /// Data that is used to build the pipeline
    /// This is a seperate struct to take advantage of Default trait derivation
    desc: PipelineDescriptor<'a>,

    /// SPIR-V bytes for the vertex shader
    vertex: ShaderModuleDescriptor<'a>,
    /// SPIR-V bytes for the fragment shader.
    /// This is optional
    fragment: Option<ShaderModuleDescriptor<'a>>,
    vertex_entry: &'a str,
    fragment_entry: &'a str,
    fragment_targets: &'a [wgpu::ColorTargetState],
}

#[derive(Default)]
struct PipelineDescriptor<'a> {
    // PIPELINE LAYOUT
    /// Bind groups that this pipeline uses. The first entry will provide all the bindings for
    /// "set = 0", second entry will provide all the bindings for "set = 1" etc.
    bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    /// Requires [`Features::PUSH_CONSTANTS`].
    push_constant_ranges: &'a [wgpu::PushConstantRange],
    // RENDER PIPELINE
    /// Primitive type the input mesh is composed of. Has Default.
    primitive: wgpu::PrimitiveState,
    /// Describes the depth/stencil state in a render pipeline. Optional.
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
}
impl PipelineBuilder<'_> {
    pub fn make_spirv(bytes: &[u8]) -> Result<ShaderSource, GpuError> {
        // HACK: This is a workaround for wgpu's spirv parsing. It will panic if the bytes
        // are not valid SPIR-V instead of returning a Result.
        // But even using catch_unwind the panic will be logged in stdout. So we're
        // registering a custom panic hook to suppress the output for this function.

        // This is *potentially* dangerous since make_spirv() could panic for other reasons.
        // TODO: Check the data length and magic number here before calling make_spirv().
        // That will allow us to remove the panic code.

        // First we save the current hook
        let prev_hook = std::panic::take_hook();
        // Now we register our own hook which does nothing
        std::panic::set_hook(Box::new(|_| {}));
        // Now we try to parse the bytes, and if it panics, we return an error instead of panicking
        let result = std::panic::catch_unwind(|| wgpu::util::make_spirv(bytes))
            .map_err(|_| GpuError::ShaderParseError);
        // Now we restore the previous hook
        std::panic::set_hook(prev_hook);
        // Return the result
        result
    }

    // FIXME: This is so scuffed
    pub fn make_spirv_owned<'f>(mut vec8: Vec<u8>) -> Result<ShaderSource<'f>, GpuError> {
        // I copy-pasted this code from StackOverflow without reading the answer
        // surrounding it that told me to write a comment explaining why this code
        // is actually safe for my own use case.
        let vec32 = unsafe {
            let ratio = std::mem::size_of::<u8>() / std::mem::size_of::<u32>();

            let length = vec8.len() * ratio;
            let capacity = vec8.capacity() * ratio;
            let ptr = vec8.as_mut_ptr() as *mut u32;

            // Don't run the destructor for vec32
            std::mem::forget(vec8);

            // Construct new Vec
            Vec::from_raw_parts(ptr, length, capacity)
        };
        Ok(ShaderSource::SpirV(Cow::Owned(vec32)))
    }

    pub fn make_wgsl(wgsl: &str) -> Result<ShaderSource, GpuError> {
        Ok(ShaderSource::Wgsl(Cow::Borrowed(wgsl)))
    }

    pub fn make_wgsl_owned<'f>(wgsl: String) -> Result<ShaderSource<'f>, GpuError> {
        Ok(ShaderSource::Wgsl(Cow::Owned(wgsl)))
    }

    /// 'a: lifetime of the shader source
    /// 'b: lifetime of the input path
    pub fn shader_auto_load<'a, 'b>(path: &'b str) -> Result<ShaderSource<'a>, GpuError> {
        if let Ok(spirv) = Self::make_spirv_owned(std::fs::read(path).unwrap()) {
            Ok(spirv)
        } else if let Ok(wgsl) = Self::make_wgsl_owned(std::fs::read_to_string(path).unwrap()) {
            Ok(wgsl)
        } else {
            Err(GpuError::ShaderParseError)
        }
    }

    pub fn shader_auto(bytes: &[u8]) -> Result<ShaderSource, GpuError> {
        if let Ok(spirv) = Self::make_spirv(bytes) {
            Ok(spirv)
        } else if let Ok(wgsl) = Self::make_wgsl(Self::str_from_bytes(bytes)?) {
            Ok(wgsl)
        } else {
            Err(GpuError::ShaderParseError)
        }
    }
}
impl<'a> PipelineBuilder<'a> {
    pub fn new(gpu: GpuHandle, label: &'a str) -> Self {
        let vertex = wgpu::util::make_spirv(include_bytes!("../../shader/screen.vert.spv"));
        let fragment = wgpu::util::make_spirv(include_bytes!("../../shader/uv.frag.spv"));

        let vertex = ShaderModuleDescriptor {
            label: Some("Default vertex shader"),
            source: vertex,
        };
        let fragment = Some(ShaderModuleDescriptor {
            label: Some("Default fragment shader"),
            source: fragment,
        });

        Self {
            gpu,
            label: Some(label),
            desc: PipelineDescriptor::default(),
            vertex,
            fragment,
            vertex_entry: "main",
            fragment_entry: "main",
            fragment_targets: &[wgpu::ColorTargetState {
                format: crate::DEFAULT_SWAP_CHAIN_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }
        .cull_back()
    }
    /// Set the vertex buffer layouts
    pub fn with_vertex_layouts(mut self, layouts: &'a [wgpu::VertexBufferLayout<'a>]) -> Self {
        self.desc.vertex_layouts = layouts;
        self
    }

    fn str_from_bytes(bytes: &[u8]) -> Result<&str, GpuError> {
        std::str::from_utf8(bytes).map_err(|_| GpuError::ShaderParseError)
    }

    /// Load the vertex shader from file path.
    /// See `with_vertex()` for loading static bytes.
    pub fn load_vertex(mut self, path: &'a str) -> Self {
        self.vertex.source = Self::shader_auto_load(path).expect("Load vertex shader");
        self
    }
    /// Load the vertex shader from bytes.
    /// This is convenient for static bytes. If you want to load from a file, at
    /// runtime, see load_vertex()
    pub fn with_vertex(mut self, bytes: &'a [u8]) -> Self {
        self.vertex.source = Self::shader_auto(bytes).expect("Parse vertex shader");
        self
    }

    /// Load the fragment shader from bytes.
    /// This is convenient for static bytes. If you want to load from a file, at
    /// runtime, see load_fragment()
    pub fn with_fragment(mut self, bytes: &'static [u8]) -> Self {
        self.fragment = Some(ShaderModuleDescriptor {
            label: Some("Default fragment shader"),
            source: Self::shader_auto(bytes).expect("Parse fragment shader"),
        });
        self
    }

    /// Convenience method for with_vertex() + with_fragment()
    /// This also sets the entry points to vs_main and fs_main respectively.
    pub fn with_vertex_fragment(mut self, bytes: &'static [u8]) -> Self {
        self.vertex_entry = "vs_main";
        self.fragment_entry = "fs_main";
        self.with_vertex(bytes).with_fragment(bytes)
    }

    /// Optional version of with_fragment_bytes(), for use in macros
    /// This has no effect if None is provided. To remove the fragment shader,
    /// use no_fragment() instead.
    pub fn with_fragment_opt(self, fragment_bytes: Option<&'static [u8]>) -> Self {
        if let Some(bytes) = fragment_bytes {
            self.with_fragment(bytes)
        } else {
            self
        }
    }

    /// Load the fragment shader from file path at runtime.
    /// See `with_fragment()` for loading static bytes.
    pub fn load_fragment(mut self, fragment: &'a str) -> Self {
        self.fragment = Some(ShaderModuleDescriptor {
            label: Some("Default fragment shader"),
            source: Self::shader_auto_load(fragment).expect("Load fragment shader"),
        });
        self
    }

    pub fn with_bind_groups(mut self, bind_groups: &'a [&wgpu::BindGroupLayout]) -> Self {
        self.desc.bind_group_layouts = bind_groups;
        self
    }

    /// Cull front faces.
    /// Front is CCW.
    pub fn cull_front(mut self) -> Self {
        self.desc.primitive.cull_mode = Some(wgpu::Face::Front);
        self
    }

    /// Cull back faces.
    /// Back is CW.
    pub fn cull_back(mut self) -> Self {
        self.desc.primitive.cull_mode = Some(wgpu::Face::Back);
        self
    }

    #[must_use]
    pub fn create(&self) -> RenderPipeline {
        // Create vertex module
        let vertex_module = self.gpu.device.create_shader_module(&self.vertex);

        // Create shader module
        let fragment_module = self
            .fragment
            .as_ref()
            .map(|fragment| self.gpu.device.create_shader_module(fragment));

        // Map fragment state if Some() otherwise it is None
        let fragment = fragment_module
            .as_ref()
            .map(|fs_module| wgpu::FragmentState {
                module: fs_module,
                entry_point: self.fragment_entry,
                targets: self.fragment_targets,
            });

        // The pipeline layout
        let layout = self
            .gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: self.label_suffix("pipeline layout").as_deref(),
                bind_group_layouts: self.desc.bind_group_layouts,
                push_constant_ranges: self.desc.push_constant_ranges,
            });

        // Create the pipeline
        let pipeline = self
            .gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&layout),
                label: self.label,
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: self.vertex_entry,
                    buffers: self.desc.vertex_layouts,
                },
                primitive: self.desc.primitive,
                depth_stencil: self.desc.depth_stencil.clone(),
                multisample: self.desc.multisample,
                fragment,
            });
        RenderPipeline {
            gpu: self.gpu.clone(),
            inner: pipeline,
        }
    }

    /// Helper function to append a suffix to the label, if Some
    fn label_suffix(&self, suffix: &str) -> Option<String> {
        self.label.map(|label| format!("{} {}", label, suffix))
    }
}
