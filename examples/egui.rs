// ! Usage will be greatly improved in the future...

use std::time::{Duration, Instant};

use agpu::prelude::*;
use egui::plot::{Line, Plot, Value, Values};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let framerate = 60.0;
    // Initialize winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = egui_winit::State::new(&window);

    // Initialize the gpu
    let gpu = Gpu::builder()
        .with_label("Example Gpu Handle")
        .with_backends(wgpu::Backends::VULKAN)
        .with_profiler()
        .build(&window)
        .unwrap();

    // Create the viewport
    let viewport = gpu.new_viewport(window).create();

    let mut last_update_inst = Instant::now();

    let pipeline = gpu
        .new_pipeline("")
        .with_fragment(include_bytes!("shader/dog.frag.spv"))
        .with_bind_groups(&[])
        .create();

    let mut egui_ctx = agpu::egui::Egui::new(gpu.clone(), viewport.width(), viewport.height());

    // let mut stat_counts = [0; 5];
    let mut timestamps: Vec<(String, f32)> = vec![];

    // Start the event loop
    event_loop.run(move |event, _, control_flow| {
        // Reset control flow
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                if state.on_event(&egui_ctx, &event) {
                    return;
                };

                match event {
                    WindowEvent::Resized(new_size) => {
                        viewport.resize(new_size.width, new_size.height);
                        egui_ctx.resize(new_size.width, new_size.height)
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                viewport.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // UI logic
                let output = egui_ctx.run(state.take_egui_input(&viewport.window), |ui| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame {
                            margin: egui::vec2(100.0, 100.0),
                            ..egui::Frame::none()
                        })
                        .show(&ui, |ui| {
                            egui::SidePanel::left("dog")
                                .frame(egui::Frame {
                                    margin: egui::vec2(40.0, 40.0),
                                    corner_radius: 4.0,
                                    fill: egui::Color32::from_rgb(4, 4, 4),
                                    ..egui::Frame::none()
                                })
                                .show_inside(ui, |ui| {
                                    egui::Grid::new("stats grid").show(ui, |ui| {
                                        // // Shows stat counts
                                        // FIXME: Stat counts are broken and cause gpu to hang
                                        // for (count, label) in stat_counts
                                        //     .iter()
                                        //     .zip(thermite_core::PIPELINE_STATISTICS_LABELS)
                                        // {
                                        //     ui.label(label);
                                        //     ui.label(count);
                                        //     ui.end_row();
                                        // }

                                        for (label, value) in &timestamps {
                                            ui.label(label);
                                            ui.label(value.to_string() + " ms");
                                            ui.end_row();
                                        }
                                    });
                                    ui.add(egui::Label::new("Hello World!"));
                                    ui.label("A shorter and more convenient way to add a label.");
                                    if ui.button("Click me").clicked() { /* take some action here */
                                    }
                                })
                        });

                    // Window
                    egui::Window::new("dog").resizable(true).show(&ui, |ui| {
                        ui.add(egui::Label::new("Hello World!"));
                        ui.heading("im a dog");
                        ui.label("A shorter and more convenient way to add a label.");
                        if ui.button("Click me").clicked() { /* take some action here */ }

                        let sin = (0..1000).map(|i| {
                            let x = i as f64 * 0.01;
                            Value::new(x, x.sin())
                        });
                        let line = Line::new(Values::from_values_iter(sin));

                        // performance plotter
                        Plot::new("Performance plot").show(ui, |plot| {
                            plot.line(line);
                        });
                    });
                });

                state.handle_output(&viewport.window, &egui_ctx, output);

                // Render gpu
                let mut frame = match viewport.begin_frame() {
                    Ok(frame) => frame,
                    Err(_) => return,
                };

                // FIXME: ERROR Vulkan validation error, VK_IMAGE_LAYOUT_UNDEFINED
                // * This ERROR only happens in our example and not when used in our project

                // gpu.profiler.timestamp("begin", &mut encoder);
                // let a = viewport.depth_view.borrow();
                {
                    // Begin render pass
                    let mut render_pass = frame.render_pass("example pass").begin();

                    // Draw the scene
                    render_pass.set_pipeline(&pipeline);
                    render_pass.draw(0..3, 0..1);
                }

                {
                    let mut ui_pass = frame.render_pass("UI Render Pass").begin();

                    egui_ctx.draw(&mut ui_pass, viewport.width(), viewport.height());
                }

                // if let Ok(stats) = gpu.total_statistics() {
                //     stat_counts = stats;
                // };
                timestamps = gpu.timestamp_report();
            }

            Event::RedrawEventsCleared => {
                if let Some(instant) = next_frame_time(framerate, &mut last_update_inst) {
                    *control_flow = ControlFlow::WaitUntil(instant);
                } else {
                    viewport.request_redraw();
                }
            }
            _ => {}
        }
    });
}

fn next_frame_time(framerate: f32, last_update_inst: &mut Instant) -> Option<Instant> {
    // Clamp to some max framerate to avoid busy-looping too much (we might be in
    // wgpu::PresentMode::Mailbox, thus discarding superfluous frames)
    let target_frametime = Duration::from_secs_f32(1.0 / framerate);
    let time_since_last_frame = last_update_inst.elapsed();

    if time_since_last_frame >= target_frametime {
        *last_update_inst = Instant::now();
        None
    } else {
        Some(Instant::now() + target_frametime - time_since_last_frame)
    }
}
