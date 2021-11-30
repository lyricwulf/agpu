use agpu::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), agpu::BoxError> {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let gpu = Gpu::builder().build(&window)?;

    let viewport = gpu.new_viewport(window).create();

    let pipeline = gpu.new_pipeline("").create();

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            let mut frame = match viewport.begin_frame() {
                Ok(frame) => frame,
                Err(err) => {
                    tracing::error!("{}", err);
                    return;
                }
            };

            {
                let mut rpass = frame.render_pass("Base render pass").begin();
                rpass.set_pipeline(&pipeline);
                rpass.draw(0..3, 0..1);
            }
        }
        Event::MainEventsCleared => {
            viewport.window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    });
}
