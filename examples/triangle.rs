fn main() -> Result<(), agpu::BoxError> {
    let program = agpu::GpuProgram::builder("Triangle").build()?;
    let gpu = program.gpu.clone();

    let example_pipeline = program
        .gpu
        .new_pipeline("Example pipeline")
        .with_vertex_fragment(include_bytes!("shader/triangle.wgsl"))
        .create();

    program.run_draw(move |frame| {
        let mut encoder = gpu.create_command_encoder("Example encoder");
        // Create the render pass
        let mut render_pass = encoder
            .render_pass("Example pass", &[frame.attach_render()])
            .begin();
        // Set the pipeline and render
        render_pass.set_pipeline(&example_pipeline).draw_triangle();
    })
}
