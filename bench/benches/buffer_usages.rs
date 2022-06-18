use agpu::RenderAttachmentBuild;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn test_benches(c: &mut Criterion) {
    let gpu = agpu::Gpu::builder().build_headless().unwrap();

    let strict_buffer = gpu
        .new_buffer("strict_buffer")
        .as_uniform_buffer()
        .allow_copy_to()
        .create(&[0u8; 65536]);
    let loose_buffer = gpu
        .new_buffer("loose_buffer")
        .with_usage(agpu::wgpu::BufferUsages::all())
        .create(&[0u8; 65536]);

    let strict_bind_group = gpu.create_bind_group(&[strict_buffer.bind()]);
    let loose_bind_group = gpu.create_bind_group(&[loose_buffer.bind()]);

    let pipeline = gpu
        .new_pipeline("bench_pipeline_switching")
        .with_bind_groups(&[&strict_bind_group.layout])
        .create();

    let output = gpu
        .new_texture("bench_texture")
        .with_format(agpu::TextureFormat::Bgra8UnormSrgb)
        .as_render_target()
        .create_empty((2048, 2048));

    // c.bench_function("no_render", |b| {
    //     b.iter(|| {
    //         let mut encoder = gpu.create_command_encoder("bench_encoder");
    //         {
    //             let _r = encoder
    //                 .render_pass("bench_pass", &[output.attach_render().clear()])
    //                 .begin();
    //         }

    //         gpu.queue.submit([encoder.finish()]);
    //         gpu.device.poll(agpu::wgpu::Maintain::Wait);
    //     })
    // });

    c.bench_function("loose_bind_group", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            gpu.queue.write_buffer(&loose_buffer, 0, &[255u8; 65536]);
            {
                let mut r = encoder
                    .render_pass("bench_pass", &[output.attach_render().clear()])
                    .begin();

                r.set_pipeline(&pipeline)
                    .set_bind_group(0, &loose_bind_group, &[]);

                for i in 0..100 {
                    r.draw_triangle();
                }
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });

    c.bench_function("strict_bind_group", |b| {
        b.iter(|| {
            let mut encoder = gpu.create_command_encoder("bench_encoder");
            gpu.queue.write_buffer(&strict_buffer, 0, &[255u8; 65536]);
            {
                let mut r = encoder
                    .render_pass("bench_pass", &[output.attach_render().clear()])
                    .begin();

                r.set_pipeline(&pipeline)
                    .set_bind_group(0, &strict_bind_group, &[]);

                for i in 0..100 {
                    r.draw_triangle();
                }
            }

            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(agpu::wgpu::Maintain::Wait);
        })
    });
}
criterion_group!(benches, test_benches);
criterion_main!(benches);
