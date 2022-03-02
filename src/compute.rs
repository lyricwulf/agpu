use std::borrow::Cow;

use crate::*;

pub struct ComputePipelineBuilder<'a> {
    pub(crate) gpu: Gpu,
    pub label: Option<&'a str>,
    /// The layout of bind groups for this pipeline.
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub push_constant_ranges: &'a [wgpu::PushConstantRange],
    /// The compiled shader module for this stage.
    pub shader: wgpu::ShaderModuleDescriptor<'a>,
    /// The name of the entry point in the compiled shader. There must be a function that returns
    /// void with this name in the shader.
    pub entry_point: &'a str,
}

impl ComputePipelineBuilder<'_> {
    pub fn new(gpu: Gpu) -> Self {
        Self {
            gpu,
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
            shader: wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed("")),
            },
            entry_point: "main",
        }
    }

    pub fn load_compute(mut self, path: &str) -> Self {
        self.shader.source = PipelineBuilder::shader_auto_load(path).expect("Parse vertex shader");
        self
    }

    /// Helper function to append a suffix to the label, if Some
    fn label_suffix(&self, suffix: &str) -> Option<String> {
        self.label.map(|label| format!("{} {}", label, suffix))
    }
}
impl<'a> ComputePipelineBuilder<'a> {
    pub fn with_shader(mut self, bytes: &'a [u8]) -> Self {
        self.shader.source = PipelineBuilder::shader_auto(bytes).expect("Parse vertex shader");
        self
    }

    pub fn with_bind_groups(mut self, bind_groups: &'a [&wgpu::BindGroupLayout]) -> Self {
        self.bind_group_layouts = bind_groups;
        self
    }
}
impl ComputePipelineBuilder<'_> {
    #[must_use]
    pub fn create<'f>(self) -> ComputePipeline<'f> {
        // Create compute module
        let cs_module = self.gpu.device.create_shader_module(&self.shader);

        // The pipeline layout
        let layout = self
            .gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: self.label_suffix("pipeline layout").as_deref(),
                bind_group_layouts: self.bind_group_layouts,
                push_constant_ranges: self.push_constant_ranges,
            });

        // Create the pipeline
        let pipeline = self
            .gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: self.label_suffix("compute pipeline").as_deref(),
                layout: Some(&layout),
                module: &cs_module,
                entry_point: self.entry_point,
            });
        ComputePipeline {
            gpu: self.gpu,
            inner: pipeline,
            bind_groups: &[],
            push_constants: &[],
        }
    }

    pub fn create_with_bindings_impl<'b, 'f>(
        self,
        layouts: &'b [&'b wgpu::BindGroupLayout],
    ) -> ComputePipeline<'f> {
        // Create compute module
        let cs_module = self.gpu.device.create_shader_module(&self.shader);

        // The pipeline layout
        let layout = self
            .gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: self.label_suffix("pipeline layout").as_deref(),
                bind_group_layouts: layouts,
                push_constant_ranges: self.push_constant_ranges,
            });

        // Create the pipeline
        let pipeline = self
            .gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: self.label_suffix("compute pipeline").as_deref(),
                layout: Some(&layout),
                module: &cs_module,
                entry_point: self.entry_point,
            });
        ComputePipeline {
            gpu: self.gpu,
            inner: pipeline,
            bind_groups: &[],
            push_constants: &[],
        }
    }

    #[must_use]
    pub fn create_with_bindings<'f>(self, bindings: &'f [&'f BindGroup]) -> ComputePipeline<'f> {
        let bind_group_layouts = bindings
            .iter()
            .map(|b| b.layout.inner())
            .collect::<Vec<_>>();

        let pipeline = self.create_with_bindings_impl(&bind_group_layouts);
        ComputePipeline {
            bind_groups: bindings,
            ..pipeline
        }
    }
}

impl Gpu {
    pub fn new_compute(&self) -> ComputePipelineBuilder {
        ComputePipelineBuilder::new(self.clone())
    }
}

pub struct ComputePipeline<'a> {
    /// We store the gpu handle so we can do standalone compute passes
    pub(crate) gpu: Gpu,
    pub inner: wgpu::ComputePipeline,
    pub bind_groups: &'a [&'a BindGroup],
    pub push_constants: &'a [u8],
}
wgpu_inner_deref!(ComputePipeline<'_>, ComputePipeline);

impl ComputePipeline<'_> {
    pub fn dispatch_pass<'a, 'f>(&'a self, c_pass: &'f mut wgpu::ComputePass<'a>, dims: &[u32]) {
        // Set pipeline
        c_pass.set_pipeline(&self.inner);
        // Set bind groups
        for (i, b) in self.bind_groups.iter().enumerate() {
            c_pass.set_bind_group(i as _, &b.inner, &[]);
        }
        // TODO: Set push constants?

        let (x, y, z) = {
            (
                *dims.get(0).unwrap_or(&1),
                *dims.get(1).unwrap_or(&1),
                *dims.get(2).unwrap_or(&1),
            )
        };

        c_pass.dispatch(x, y, z);
    }

    pub fn dispatch_encoder<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, dims: &[u32]) {
        let mut c = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Standalone Compute Pass"),
        });

        self.dispatch_pass(&mut c, dims);
    }

    pub fn dispatch(&self, dims: &[u32]) {
        let mut encoder = self
            .gpu
            .create_command_encoder("Standalone Compute Dispatch Encoder");

        self.dispatch_encoder(&mut encoder, dims);
    }
}
