// ! Usage will be greatly improved in the future...

use std::mem::size_of;
use std::time::{Duration, Instant};

use agpu::prelude::*;
use egui::plot::{Line, Plot, Value, Values};
use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use egui::epaint;

fn main() {
    tracing_subscriber::fmt::init();

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

    let mut egui_ctx = egui::CtxRef::default();
    // let mut stat_counts = [0; 5];
    let mut timestamps: Vec<(String, f32)> = vec![];

    let vertex_layout = wgpu::VertexBufferLayout {
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
        array_stride: (2 + 2 + 1) * 4,
    };

    let mut vertex_buffers = Vec::<(Buffer, Buffer)>::new();

    egui_ctx.begin_frame(Default::default());
    let egui_font_texture = egui_ctx.texture();

    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("UI sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // font data
    // we need to convert the texture into rgba_srgb format
    let mut pixels: Vec<u8> = Vec::with_capacity(egui_font_texture.pixels.len() * 4);
    for srgba in egui_font_texture.srgba_pixels(0.33) {
        pixels.push(srgba.r());
        pixels.push(srgba.g());
        pixels.push(srgba.b());
        pixels.push(srgba.a());
    }

    let font_texture = gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("EGUI font texture"),
            size: wgpu::Extent3d {
                width: egui_font_texture.width as u32,
                height: egui_font_texture.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        },
        &pixels,
    );
    let font_texture_view = font_texture.create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    });

    let bind_group_layout = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });
    let ui_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("UI bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    viewport.data_buffer.as_entire_buffer_binding(),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&font_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let ui_pipeline = gpu
        .new_pipeline("")
        .with_vertex_layouts(&[vertex_layout])
        .with_fragment(include_bytes!("shader/egui.frag.spv"))
        .with_vertex(include_bytes!("shader/egui.vert.spv"))
        .with_bind_groups(&[&bind_group_layout])
        .create();

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
                    WindowEvent::Resized(_) => {
                        // viewport.resize(new_size.width, new_size.height);
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
                egui_ctx.begin_frame(state.take_egui_input(&viewport.window));
                egui::CentralPanel::default()
                    .frame(egui::Frame {
                        margin: egui::vec2(100.0, 100.0),
                        ..egui::Frame::none()
                    })
                    .show(&egui_ctx, |ui| {
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
                                if ui.button("Click me").clicked() { /* take some action here */ }
                            })
                    });

                // Window
                egui::Window::new("dog")
                    .resizable(true)
                    .show(&egui_ctx, |ui| {
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
                        ui.add(Plot::new("Performance plot").line(line));
                    });

                let (output, cl_sh) = egui_ctx.end_frame();

                state.handle_output(&viewport.window, &egui_ctx, output);

                let cl_me = egui_ctx.tessellate(cl_sh);

                // update ui buffers
                for (i, egui::ClippedMesh(_, mesh)) in cl_me.iter().enumerate() {
                    // create any missing buffers
                    if i >= vertex_buffers.len() {
                        vertex_buffers.push((
                            gpu.new_buffer("")
                                .as_vertex_buffer()
                                .allow_copy_to()
                                .create(&mesh.vertices),
                            gpu.new_buffer("")
                                .as_index_buffer()
                                .allow_copy_to()
                                .create(&mesh.indices),
                        ));
                    } else {
                        // resize buffer if needed
                        if size_of::<epaint::Vertex>() * mesh.vertices.len()
                            > vertex_buffers[i].0.size()
                        {
                            vertex_buffers[i].0 = gpu
                                .new_buffer("")
                                .as_vertex_buffer()
                                .allow_copy_to()
                                .create(&mesh.vertices);
                        } else {
                            gpu.queue.write_buffer(
                                &vertex_buffers[i].0,
                                0,
                                bytemuck::cast_slice(&mesh.vertices),
                            );
                        };

                        if size_of::<u32>() * mesh.indices.len() > vertex_buffers[i].1.size() {
                            vertex_buffers[i].1 = gpu
                                .new_buffer("")
                                .as_index_buffer()
                                .allow_copy_to()
                                .create(&mesh.indices);
                        } else {
                            gpu.queue.write_buffer(
                                &vertex_buffers[i].1,
                                0,
                                bytemuck::cast_slice(&mesh.indices),
                            );
                        }
                    }
                }
                // Submit buffer updates
                gpu.queue.submit(None);

                // Render gpu
                let mut frame = viewport.begin_frame().unwrap();

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

                    ui_pass.set_pipeline(&ui_pipeline);
                    ui_pass.set_bind_group(0, &ui_bind_group, &[]);

                    for (egui::ClippedMesh(clip, me), (vb, ib)) in cl_me.iter().zip(&vertex_buffers)
                    {
                        if let Some((x, y, width, height)) = render_region(clip, &viewport) {
                            ui_pass.set_scissor_rect(x, y, width, height);
                            ui_pass.set_vertex_buffer(0, vb.slice(..));
                            ui_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                            ui_pass.draw_indexed(0..me.indices.len() as u32, 0, 0..1);
                        }
                    }
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

/// Uses https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/lib.rs
fn render_region(clip_rect: &egui::Rect, viewport: &Viewport) -> Option<(u32, u32, u32, u32)> {
    let scale_factor = 1.0;
    // Transform clip rect to physical pixels.
    let clip_min_x = scale_factor * clip_rect.min.x;
    let clip_min_y = scale_factor * clip_rect.min.y;
    let clip_max_x = scale_factor * clip_rect.max.x;
    let clip_max_y = scale_factor * clip_rect.max.y;

    // Make sure clip rect can fit within an `u32`.
    let clip_min_x = clip_min_x.clamp(0.0, viewport.width() as f32);
    let clip_min_y = clip_min_y.clamp(0.0, viewport.height() as f32);
    let clip_max_x = clip_max_x.clamp(clip_min_x, viewport.width() as f32);
    let clip_max_y = clip_max_y.clamp(clip_min_y, viewport.height() as f32);

    let clip_min_x = clip_min_x.round() as u32;
    let clip_min_y = clip_min_y.round() as u32;
    let clip_max_x = clip_max_x.round() as u32;
    let clip_max_y = clip_max_y.round() as u32;

    let width = (clip_max_x - clip_min_x).max(1);
    let height = (clip_max_y - clip_min_y).max(1);

    // Clip scissor rectangle to target size.
    let x = clip_min_x.min(viewport.width());
    let y = clip_min_y.min(viewport.height());
    let width = width.min(viewport.width() - x);
    let height = height.min(viewport.height() - y);

    // Skip rendering with zero-sized clip areas.
    if width == 0 || height == 0 {
        return None;
    }

    Some((x, y, width, height))
}
