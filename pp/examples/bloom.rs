use agpu::{prelude::*, GpuProgram};
use agpu_pp::bloom::Bloom;

fn main() -> Result<(), BoxError> {
    println!("Hello, world!");
    let mut program = GpuProgram::builder("Bloom example").with_framerate(42069999.0);
    program.gpu = program.gpu.with_backends(agpu::Backends::VULKAN);
    let program = program.build()?;
    let gpu = program.gpu.clone();
    let example_pipeline = program
        .gpu
        .new_pipeline("Example pipeline")
        .with_vertex_fragment(include_bytes!("shader/triangle.wgsl"))
        .create();
    let mut bloom = Bloom::new(&gpu, 1280, 720, program.viewport.sc_desc.borrow().format);

    program.run_draw(move |mut frame| {
        if let Some((width, height)) = frame.resized_to {
            bloom.resize(width, height);
        }

        frame
            .render_pass("bloom pass")
            .with_pipeline(&example_pipeline)
            .begin()
            .draw_triangle();

        bloom.apply(&frame.view, &mut frame.encoder)
    });
}
