use tracing::warn;
pub use wgpu::CompareFunction;

use std::borrow::Cow;

use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;

use crate::GpuError;
use crate::GpuHandle;

use crate::RenderPipeline;

pub trait ColorTargetBuilderExt {
    fn blend_over(self) -> Self;
    fn blend_over_premult(self) -> Self;
    fn blend_add(self) -> Self;
    fn blend_subtract(self) -> Self;
    fn write_mask(self, mask: u32) -> Self;
}
impl ColorTargetBuilderExt for wgpu::ColorTargetState {
    fn blend_over(mut self) -> Self {
        self.blend = Some(wgpu::BlendState::ALPHA_BLENDING);
        self
    }
    fn blend_over_premult(mut self) -> Self {
        self.blend = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        self
    }

    fn blend_add(mut self) -> Self {
        self.blend = Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::SrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        });
        self
    }

    fn blend_subtract(mut self) -> Self {
        self.blend = Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::SrcAlpha,
                operation: wgpu::BlendOperation::Subtract,
            },
            alpha: wgpu::BlendComponent::OVER,
        });
        self
    }

    fn write_mask(mut self, mask: u32) -> Self {
        self.write_mask = wgpu::ColorWrites::from_bits(mask).unwrap();
        self
    }
}

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
        const DEFAULT_FRAGMENT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

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
                // TODO: Use gpu.preferred_format
                format: DEFAULT_FRAGMENT_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }
    }

    /// Set the label
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the vertex buffer layouts
    pub fn with_vertex_layouts(mut self, layouts: &'a [wgpu::VertexBufferLayout<'a>]) -> Self {
        self.desc.vertex_layouts = layouts;
        self
    }

    /// Set the fragment layouts
    pub fn with_fragment_targets(mut self, targets: &'a [wgpu::ColorTargetState]) -> Self {
        self.fragment_targets = targets;
        self
    }

    /// Declare a depth state for the pipeline. MUST be called if the pipeline is
    /// set for a render pass with depth attachment
    pub fn with_depth(mut self) -> Self {
        self.desc.depth_stencil = Some(wgpu::DepthStencilState {
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            format: wgpu::TextureFormat::Depth32Float,
            bias: wgpu::DepthBiasState::default(),
        });
        self
    }

    fn do_depth<F>(&mut self, op: F)
    where
        F: FnOnce(&mut wgpu::DepthStencilState),
    {
        if let Some(desc) = self.desc.depth_stencil.as_mut() {
            op(desc);
        } else {
            warn!("Depth mod was called before with_depth() was called in pipeline builder");
        }
    }

    /// Add a constant depth biasing factor, in basic units of the depth format.
    /// Add a slope depth biasing factor.
    /// TODO: Clarify what this means??
    pub fn depth_bias(mut self, constant: i32, slope: f32) -> Self {
        self.do_depth(|desc| {
            desc.bias.constant = constant;
            desc.bias.slope_scale = slope;
        });
        self
    }
    /// Add a depth bias clamp value (absolute).
    pub fn depth_bias_clamp(mut self, clamp: f32) -> Self {
        self.do_depth(|desc| {
            desc.bias.clamp = clamp;
        });
        self
    }

    /// Set the depth comparison function
    /// Values testing `true` will pass the depth test
    pub fn depth_compare(mut self, compare: CompareFunction) -> Self {
        self.do_depth(|desc| {
            desc.depth_compare = compare;
        });
        self
    }

    pub fn with_depth_stencil(mut self) -> Self {
        self.desc.depth_stencil = Some(wgpu::DepthStencilState {
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            // TODO: Actually need a stencil state to use stencil lol
            stencil: wgpu::StencilState::default(),
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            bias: wgpu::DepthBiasState::default(),
        });
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

    pub fn with_fragment_entry(mut self, entry: &'a str) -> Self {
        self.fragment_entry = entry;
        self
    }

    pub fn with_vertex_entry(mut self, entry: &'a str) -> Self {
        self.vertex_entry = entry;
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

    /// Draws lines instead of filling in triangles.
    pub fn wireframe(mut self) -> Self {
        self.desc.primitive.polygon_mode = wgpu::PolygonMode::Line;
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

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
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
            // TODO: Implement multiview interface
            multiview: None,
        };

        // Create the pipeline
        let pipeline = self.gpu.device.create_render_pipeline(&pipeline_desc);
        RenderPipeline {
            depth_stencil: self.desc.depth_stencil.clone(),
            gpu: self.gpu.clone(),
            inner: pipeline,
        }
    }

    /// Helper function to append a suffix to the label, if Some
    fn label_suffix(&self, suffix: &str) -> Option<String> {
        self.label.map(|label| format!("{} {}", label, suffix))
    }
}
