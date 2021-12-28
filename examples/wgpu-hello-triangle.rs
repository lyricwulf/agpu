const GREEN: u32 = 0x00FF00FF;

fn main() -> Result<(), agpu::BoxError> {
    let mut program = agpu::GpuProgram::builder("Triangle");
    program.gpu = program.gpu.with_backends(agpu::Backends::GL);
    let program = match program.build() {
        Ok(program) => program,
        Err(agpu::GpuError::RequestDeviceError(e)) => {
            panic!("{}", e);
        }
        Err(err) => {
            panic!("{}", err);
        }
    };

    let example_pipeline = program
        .gpu
        .new_pipeline("Example pipeline")
        .with_vertex_fragment(include_bytes!("shader/hello-triangle.wgsl"))
        .create();

    program.run_draw(move |mut frame| {
        // Create the render pass
        let mut render_pass = frame.render_pass_cleared("Triangle pass", GREEN).begin();
        // Set the pipeline and render
        render_pass.set_pipeline(&example_pipeline).draw_triangle();
    })
}
