use agpu::prelude::*;
use bytemuck::{Pod, Zeroable};

const BABY_BLUE: u32 = 0x20_40_60_FF;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, VertexLayout)]
struct Vertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

fn generate_matrix(aspect_ratio: f32) -> nalgebra::Matrix4<f32> {
    let mx_projection = nalgebra::Perspective3::new(aspect_ratio, 45.0_f32.to_radians(), 1.0, 10.0);
    let mx_view = nalgebra::Matrix4::look_at_rh(
        &[1.5, -5.0, 3.0].into(),
        &[0.0, 0.0, 0.0].into(),
        &nalgebra::Vector3::z(),
    );
    mx_projection.to_homogeneous() * mx_view
}

fn main() -> Result<(), BoxError> {
    // Init gpu
    let program = agpu::GpuProgram::builder("Cube example")
        .with_gpu_features(Features::POLYGON_MODE_LINE)
        .build()?;
    let gpu = program.gpu.clone();

    let (vertex_data, index_data) = create_vertices();

    // Create the vertex buffer
    let vertex_buffer = gpu
        .new_buffer("Vertex buffer")
        .as_vertex_buffer()
        .create(&vertex_data);
    let index_buffer = gpu
        .new_buffer("Index buffer")
        .as_index_buffer()
        .create(&index_data);

    // Create the texture
    let size = 256u32;
    let texels = create_texels(size as usize);
    let texture = gpu
        .new_texture("Texture")
        .with_format(TextureFormat::R8Uint)
        .allow_binding()
        .create(&texels, &[size, size]);

    // Create other resources
    let mx = generate_matrix(program.viewport.aspect_ratio());
    let uniform_buf = gpu
        .new_buffer("Uniform Buffer")
        .as_uniform_buffer()
        .allow_copy_to()
        .create(mx.as_ref());

    let bind_group = gpu.create_bind_group(&[
        uniform_buf.bind_ubo().in_vertex(),
        texture.bind_texture().sample_uint().in_fragment(),
    ]);

    let vertex_layouts = &[Vertex::vertex_buffer_layout::<0>()];
    let bind_groups = &[&bind_group.layout];
    let pipeline_builder = gpu
        .new_pipeline("Cube pipeline")
        .with_vertex_fragment(include_bytes!("shader/cube.wgsl"))
        .with_vertex_layouts(vertex_layouts)
        .with_bind_groups(bind_groups)
        .cull_back();
    let pipeline = pipeline_builder.create();
    let wire_pipeline = pipeline_builder
        .with_fragment_entry("fs_wire")
        .wireframe()
        .create();

    program.on_resize(move |_, width, height| {
        let mx = generate_matrix(width as f32 / height as f32);
        uniform_buf.write_unchecked(mx.as_ref());
    });

    program.run_draw(move |frame| {
        let mut encoder = frame.create_encoder("Cube encoder");
        let mut rpass = encoder
            .render_pass("Main pass", &[frame.attach_render().clear_color(BABY_BLUE)])
            .begin();
        rpass
            .set_vertex_buffer(0, vertex_buffer.slice(..))
            .set_index_buffer(index_buffer.slice(..))
            .set_bind_group(0, &bind_group.inner, &[])
            .set_pipeline(&pipeline)
            .draw_one_indexed(index_data.len() as _);
        rpass
            .set_pipeline(&wire_pipeline)
            .draw_one_indexed(index_data.len() as _);
    });
}
