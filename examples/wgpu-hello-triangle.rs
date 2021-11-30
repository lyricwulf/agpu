const GREEN: u32 = 0x00FF00FF;

fn main() -> Result<(), agpu::BoxError> {
    let program = agpu::GpuProgram::builder("Triangle").build()?;

    let example_pipeline = program
        .gpu
        .new_pipeline("Example pipeline")
        .with_vertex_fragment(include_bytes!("shader/hello-triangle.wgsl"))
        .create();

    program.run_draw(move |mut frame| {
        frame
            .render_pass("Example render pass")
            .with_pipeline(&example_pipeline)
            .clear_color(GREEN)
            .begin()
            .draw_triangle();
    })
}
