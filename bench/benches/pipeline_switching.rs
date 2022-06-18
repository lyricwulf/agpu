use agpu::RenderAttachmentBuild;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn pipeline_switching(c: &mut Criterion) {
    let gpu = agpu::Gpu::builder().build_headless().unwrap();
    let pipeline = [
        gpu.new_pipeline("bench_pipeline_switching").create(),
        gpu.new_pipeline("bench_pipeline_switching_2").create(),
    ];
    let output = gpu
        .new_texture("bench_texture")
        .with_format(agpu::TextureFormat::Bgra8UnormSrgb)
        .as_render_target()
        .create_empty((2048, 2048));

    c.bench_function("no_render", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            {
                let mut r = encoder
                    .render_pass("bench_pass", &[output.attach_render().clear()])
                    .begin();
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });

    c.bench_function("one_pipeline_one_pass", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            {
                let mut r = encoder
                    .render_pass("bench_pass", &[output.attach_render().clear()])
                    .begin();

                r.set_pipeline(&pipeline[0]);

                for i in 0..100 {
                    r.draw_triangle();
                }
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });

    c.bench_function("many_pipeline_one_pass", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            {
                let mut r = encoder
                    .render_pass("bench_pass", &[output.attach_render().clear()])
                    .begin();

                for i in 0..100 {
                    r.set_pipeline(&pipeline[i % 2]).draw_triangle();
                }
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });

    c.bench_function("many_pass_one_pipeline", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            {
                for i in 0..100 {
                    let mut r = encoder
                        .render_pass("bench_pass", &[output.attach_render().clear()])
                        .begin();
                    r.set_pipeline(&pipeline[0]).draw_triangle();
                }
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });
}
criterion_group!(benches, pipeline_switching);
criterion_main!(benches);
