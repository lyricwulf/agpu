#![cfg(test)]

use agpu::{u8n, VertexLayout, VertexLayoutInstance};

#[derive(VertexLayout)]
struct MixedTypes {
    _position: [f32; 3],
    _color: [u8n; 4],
    _normal: [f32; 4],
    _some_raw_data: [u16; 4],
}

#[test]
fn vertex_layout_mixed_types() {
    // Start at shader location 3
    let layout = MixedTypes::vertex_buffer_layout::<3>();
    assert_eq!(
        layout,
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MixedTypes>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint16x4,
                },
            ],
        }
    );
}

#[derive(VertexLayoutInstance)]
struct PerInstanceData {
    _rotation: [f32; 4],
    _position: [f32; 3],
}

#[test]
fn vertex_layout_per_instance() {
    // Start at shader location 3
    let layout = PerInstanceData::vertex_buffer_layout::<7>();
    assert_eq!(
        layout,
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PerInstanceData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    );
}
