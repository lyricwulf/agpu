fn main() -> Result<(), agpu::BoxError> {
    let program = agpu::GpuProgram::builder().build()?;

    let example_pipeline = program.gpu.new_pipeline().create();

    program.run_draw(move |mut frame| {
        frame
            .render_pass("Example render pass")
            .with_pipeline(&example_pipeline)
            .begin()
            .draw_triangle();
    })
}
