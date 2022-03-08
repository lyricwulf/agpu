//! This example is currently broken because I don't understand the new egui
//! textures since 0.17

#![cfg(feature = "egui")]

use std::ops::{Deref, DerefMut};

use crate::{BindGroup, Buffer, Gpu, RenderPass, RenderPipeline, Sampler};

pub struct Egui {
    pub ctx: egui::Context,
    gpu: Gpu,
    vertex_buffers: Vec<(Buffer, Buffer)>,
    ubo_buffer: Buffer,
    _linear_sampler: Sampler,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
    last_meshes: Vec<egui::epaint::ClippedMesh>,
}
impl Deref for Egui {
    type Target = egui::Context;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl DerefMut for Egui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
impl Egui {
    const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
        array_stride: (2 + 2 + 1) * 4,
    };

    pub fn new(gpu: crate::Gpu, width: u32, height: u32) -> Self {
        // Create the egui context
        let ctx = egui::Context::default();
        // The vertex layout is a static constant

        // Create the UBO buffer
        let ubo_buffer = gpu
            .new_buffer("EGUI UBO buffer")
            .as_uniform_buffer()
            .allow_copy_to()
            .create(&[width as f32, height as f32]);

        // Create the linear sampler
        let linear_sampler = gpu
            .new_sampler("EGUI linear sampler")
            .linear_filter()
            .create();

        // Create the bind group and pipeline
        let bind_group = gpu.create_bind_group(&[
            ubo_buffer.bind_uniform(),
            linear_sampler.bind().in_fragment(),
        ]);
        let pipeline = gpu
            .new_pipeline("EGUI pipeline")
            .with_vertex_layouts(&[Self::VERTEX_LAYOUT])
            .with_fragment(include_bytes!("egui/shader/egui.frag.spv"))
            .with_vertex(include_bytes!("egui/shader/egui.vert.spv"))
            .with_bind_groups(&[&bind_group.layout])
            .create();

        Self {
            ctx,
            gpu,
            vertex_buffers: Vec::new(),
            ubo_buffer,
            _linear_sampler: linear_sampler,
            bind_group,
            pipeline,
            last_meshes: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ubo_buffer
            .write_unchecked(&[width as f32, height as f32]);
    }

    pub fn run(
        &mut self,
        new_input: egui::RawInput,
        run_ui: impl FnOnce(&egui::Context),
    ) -> egui::PlatformOutput {
        let output = self.ctx.run(new_input, run_ui);
        let meshes = self.ctx.tessellate(output.shapes);
        self.update_buffers(&meshes);
        self.last_meshes = meshes;
        output.platform_output
    }

    fn update_buffers(&mut self, meshes: &[egui::epaint::ClippedMesh]) {
        // Update the existing buffers
        for (mesh, (v_buf, i_buf)) in meshes
            .iter()
            .map(|m| &m.1)
            .zip(self.vertex_buffers.iter_mut())
        {
            v_buf.write(&mesh.vertices);
            i_buf.write(&mesh.indices);
        }

        // Create new buffers if necessary
        if self.vertex_buffers.len() < meshes.len() {
            for m in meshes.iter().map(|m| &m.1).skip(self.vertex_buffers.len()) {
                let new_buffers = self.new_vertex_buffer(&m.vertices, &m.indices);
                self.vertex_buffers.push(new_buffers);
            }
        }
    }

    fn new_vertex_buffer<T1, T2>(&self, v: &[T1], i: &[T2]) -> (Buffer, Buffer)
    where
        T1: bytemuck::Pod,
        T2: bytemuck::Pod,
    {
        (
            self.gpu
                .new_buffer("EGUI vertex buffer")
                .as_vertex_buffer()
                .allow_copy_to()
                .create(v),
            self.gpu
                .new_buffer("EGUI index buffer")
                .as_index_buffer()
                .allow_copy_to()
                .create(i),
        )
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, width: u32, height: u32) {
        // Arm the pipeline
        render_pass
            .set_pipeline(&self.pipeline)
            .set_bind_group(0, &self.bind_group, &[]);

        for (egui::ClippedMesh(clip_rect, mesh), (vb, ib)) in
            self.last_meshes.iter().zip(&self.vertex_buffers)
        {
            if let Some((x, y, width, height)) = render_region(clip_rect, width, height) {
                render_pass
                    .set_scissor_rect(x, y, width, height)
                    .set_vertex_buffer(0, vb.slice(..))
                    .set_index_buffer_u32(ib.slice(..))
                    .draw_one_indexed(mesh.indices.len() as u32);
            }
        }
    }
}

/// Uses https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/lib.rs
fn render_region(clip_rect: &egui::Rect, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let scale_factor = 1.0;
    // Transform clip rect to physical pixels.
    let clip_min_x = scale_factor * clip_rect.min.x;
    let clip_min_y = scale_factor * clip_rect.min.y;
    let clip_max_x = scale_factor * clip_rect.max.x;
    let clip_max_y = scale_factor * clip_rect.max.y;

    // Make sure clip rect can fit within an `u32`.
    let clip_min_x = clip_min_x.clamp(0.0, width as f32);
    let clip_min_y = clip_min_y.clamp(0.0, height as f32);
    let clip_max_x = clip_max_x.clamp(clip_min_x, width as f32);
    let clip_max_y = clip_max_y.clamp(clip_min_y, height as f32);

    let clip_min_x = clip_min_x.round() as u32;
    let clip_min_y = clip_min_y.round() as u32;
    let clip_max_x = clip_max_x.round() as u32;
    let clip_max_y = clip_max_y.round() as u32;

    let clip_width = (clip_max_x - clip_min_x).max(1);
    let clip_height = (clip_max_y - clip_min_y).max(1);

    // Clip scissor rectangle to target size.
    let x = clip_min_x.min(width);
    let y = clip_min_y.min(height);
    let clip_width = clip_width.min(width - x);
    let clip_height = clip_height.min(height - y);

    // Skip rendering with zero-sized clip areas.
    if width == 0 || height == 0 {
        return None;
    }

    Some((x, y, clip_width, clip_height))
}
