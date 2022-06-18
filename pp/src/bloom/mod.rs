use agpu::prelude::*;
use agpu::pub_const_flag;
use agpu::wgpu;
use bytemuck::Pod;
use bytemuck::Zeroable;

type Texture = agpu::Texture<D2>;

pub_const_flag!(SAMPLED_ATTACHMENT: wgpu::TextureUsages = RENDER_ATTACHMENT | TEXTURE_BINDING);

pub struct BoundTexture {
    pub texture: Texture,
    pub binding: BindGroup,
}

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct BloomData {
    pub screen_aspect: f32,
    pub layer_factor: f32,
}
impl Default for BloomData {
    fn default() -> Self {
        Self {
            screen_aspect: 1.0,
            layer_factor: 1.0,
        }
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct BloomSettings {
    pub aspect: f32,
    pub intensity: f32,
}
impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            aspect: 1.0,
            intensity: 1.0,
        }
    }
}

#[allow(dead_code)]
pub struct Bloom {
    gpu: Gpu,
    threshold_pipeline: RenderPipeline,
    threshold_bind_group: BindGroup,
    down_pipeline: RenderPipeline,
    up_pipeline: RenderPipeline,
    add_pipeline: RenderPipeline,
    format: TexFormat,
    width: u32,
    height: u32,
    mips: Vec<BoundTexture>,
    data: BloomData,
    data_buffer: Buffer,
    settings: BloomSettings,
    settings_buffer: Buffer,
    linear_sampler: Sampler,
    nearest_sampler: Sampler,
}
// Can be const when cmp max is const
fn tex_size(mip: u32, width: u32, height: u32) -> D2 {
    (
        (width / (2_u32.pow(mip as u32))).max(1),
        (height / (2_u32.pow(mip as u32))).max(1),
    )
}
// Following ISS #70887 we can use int_log
fn mip_count(size: u32) -> u32 {
    (size as f32).log2().ceil() as u32
}

fn create_mips(
    gpu: &Gpu,
    linear_sampler: &Sampler,
    format: TexFormat,
    width: u32,
    height: u32,
) -> Vec<BoundTexture> {
    let major_dimension = width.max(height);
    let mip_count = mip_count(major_dimension);

    (0..mip_count)
        .map(|i| {
            println!(
                "Creating mip {} at resolution {:?}",
                i,
                tex_size(i, width, height)
            );
            let texture = gpu
                .new_texture(&("Downchain mip".to_string() + &i.to_string()))
                .with_format(*format)
                .allow_binding()
                .as_render_target()
                .create_empty(tex_size(i, width, height));
            let binding = gpu.create_bind_group(&[linear_sampler.bind(), texture.bind()]);
            BoundTexture { texture, binding }
        })
        .collect::<Vec<_>>()
}

pub fn pipeline_builder<'a>(
    gpu: &Gpu,
    targets: &'a [wgpu::ColorTargetState],
    bind_group: &'a [&wgpu::BindGroupLayout],
) -> PipelineBuilder<'a> {
    gpu.new_pipeline("Bloom pipeline builder")
        .with_fragment_targets(targets)
        .with_bind_groups(bind_group)
        .with_vertex(include_bytes!("shader/screen.vert.spv"))
}

impl Bloom {
    pub fn new(gpu: &Gpu, width: u32, height: u32, format: TextureFormat) -> Self {
        let format: TexFormat = format.into();
        // Create sampler and textures
        let linear_sampler = gpu.new_sampler("Linear Sampler").linear_filter().create();
        let nearest_sampler = gpu.new_sampler("Nearest Sampler").create();
        let mips = create_mips(gpu, &linear_sampler, format, width, height);
        let threshold_bind_group =
            gpu.create_bind_group(&[nearest_sampler.bind(), mips[0].texture.bind()]);
        let binding_layout = &mips[0].binding.layout;

        let data_buffer = gpu
            .new_buffer("Bloom data buffer")
            .as_uniform_buffer()
            .create(&[BloomData::default()]);
        let settings_buffer = gpu
            .new_buffer("Bloom settings buffer")
            .as_uniform_buffer()
            .create(&[BloomSettings::default()]);

        // New usage: Construct "group" as array
        let _buffer_binding = [data_buffer.bind(), settings_buffer.bind()].create_group();

        let threshold_pipeline =
            pipeline_builder(gpu, &[format.target()], &[&threshold_bind_group.layout])
                .with_label("Bloom threshold pipeline")
                .with_fragment(include_bytes!("shader/filter-threshold.frag.spv"))
                .create();
        let down_pipeline = pipeline_builder(gpu, &[format.target()], &[binding_layout])
            .with_label("Bloom down pipeline")
            .with_fragment(include_bytes!("shader/filter-down.frag.spv"))
            .create();
        let up_pipeline = pipeline_builder(gpu, &[format.target()], &[binding_layout])
            .with_label("Bloom up pipeline")
            .with_fragment(include_bytes!("shader/filter-up.frag.spv"))
            .with_fragment_targets(&[format.target().blend_add()])
            .create();
        let add_pipeline = pipeline_builder(gpu, &[format.target()], &[binding_layout])
            .with_label("Bloom add pipeline")
            .with_fragment(include_bytes!("shader/add.frag.spv"))
            .with_fragment_targets(&[format.target().blend_add()])
            .create();

        Self {
            gpu: gpu.clone(),
            down_pipeline,
            up_pipeline,
            add_pipeline,
            width,
            height,
            mips,
            format,
            threshold_pipeline,
            threshold_bind_group,
            data: BloomData::default(),
            settings: BloomSettings::default(),
            linear_sampler,
            nearest_sampler,
            data_buffer,
            settings_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        dbg!(width, height);
        self.mips = create_mips(&self.gpu, &self.linear_sampler, self.format, width, height);
    }

    pub fn apply<'a>(&'a mut self, target: &TextureView, encoder: &mut CommandEncoder) {
        self.threshold_bind_group
            .rebind(&[self.nearest_sampler.bind(), target.bind()]);

        // Threshold pass
        self.threshold_pass(encoder, &self.threshold_bind_group);

        // Downsample along down mip chain.
        self.down_pass(encoder);

        // Upsample along up mip chain.
        self.up_pass(encoder);

        // Render to the final target
        self.add_pass(encoder, target);
    }

    fn threshold_pass(&self, encoder: &mut CommandEncoder, bind_group: &BindGroup) {
        let mut r = encoder
            .render_pass(
                "bloom threshold pass",
                &[self.mips[0].texture.attach_render().clear()],
            )
            .begin();
        r.set_pipeline(&self.threshold_pipeline);
        r.set_bind_group(0, bind_group, &[]);
        r.draw_triangle();
    }

    fn down_pass(&self, encoder: &mut CommandEncoder) {
        for (binding, texture) in self.mips[0..self.mips.len() - 1]
            .iter()
            .map(|mip| &mip.binding)
            .zip(self.mips[1..self.mips.len()].iter().map(|mip| &mip.texture))
        {
            let mut r = encoder
                .render_pass("bloom down pass", &[texture.attach_render().clear()])
                .begin();
            r.set_pipeline(&self.down_pipeline);
            r.set_bind_group(0, binding, &[]);
            r.draw_triangle();
        }
    }

    fn up_pass(&self, encoder: &mut CommandEncoder) {
        for (binding, texture) in self.mips[1..self.mips.len()]
            .iter()
            .map(|mip| &mip.binding)
            .zip(
                self.mips[0..self.mips.len() - 1]
                    .iter()
                    .map(|mip| &mip.texture),
            )
            .rev()
        {
            let mut r = encoder
                .render_pass("bloom up pass", &[texture.attach_render()])
                .begin();
            r.set_pipeline(&self.up_pipeline);
            r.set_bind_group(0, binding, &[]);
            r.draw(0..3, 0..1);
        }
    }

    fn add_pass(&self, encoder: &mut CommandEncoder, target: &TextureView) {
        let mut r = encoder
            .render_pass("bloom add pass", &[target.attach_render()])
            .begin();
        r.set_pipeline(&self.add_pipeline);
        r.set_bind_group(0, &self.mips[0].binding, &[]);
        r.draw_triangle();
    }
}
