//! An implementation of stencil using depth buffer

use agpu::prelude::*;

fn main() -> Result<(), agpu::BoxError> {
    tracing_subscriber::fmt::init();
    let program = agpu::GpuProgram::builder("Triangle").build()?;
    let gpu = program.gpu.clone();

    let example_pipeline = program
        .gpu
        .new_pipeline("Example pipeline")
        .with_vertex_fragment(include_bytes!("shader/hello-triangle.wgsl"))
        .with_depth()
        .depth_compare(agpu::wgpu::CompareFunction::Less)
        .depth_bias(0, 0.0)
        .create();

    let example_pipeline_2 = program
        .gpu
        .new_pipeline("Example pipeline 2")
        .with_vertex_fragment(include_bytes!("shader/hello-triangle.wgsl"))
        .with_vertex_entry("vs_2")
        .with_fragment_entry("fs_2")
        .with_depth()
        .depth_compare(agpu::wgpu::CompareFunction::Equal)
        .depth_bias(0, 1000.0)
        .create();

    let depth_texture = program
        .gpu
        .new_texture("Depth texture")
        .as_depth()
        .create_empty((
            program.viewport.inner_size().width,
            program.viewport.inner_size().height,
        ));

    program.run_draw(move |frame| {
        let mut encoder = gpu.create_command_encoder("Triangle pass");
        // Create the render pass
        {
            let mut render_pass = encoder
                .render_pass("Triangle pass", &[frame.attach_render().clear()])
                .with_depth(depth_texture.attach_depth().clear_depth())
                .begin();
            // Set the pipeline and render
            render_pass.set_pipeline(&example_pipeline).draw_triangle();

            render_pass
                .set_pipeline(&example_pipeline_2)
                .draw_triangle();
        }
    })
}
