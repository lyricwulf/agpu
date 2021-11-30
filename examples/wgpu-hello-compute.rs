use agpu::prelude::*;
use std::{ops::Deref, str::FromStr};

fn main() {
    let numbers = if std::env::args().len() <= 1 {
        let default = vec![1, 2, 3, 4];
        println!("No numbers were provided, defaulting to {:?}", default);
        default
    } else {
        std::env::args()
            .skip(1)
            .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
            .collect()
    };

    let steps = execute_gpu(&numbers).unwrap();

    let disp_steps: Vec<String> = steps
        .iter()
        .map(|&n| match n {
            u32::MAX => "OVERFLOW".to_string(),
            _ => n.to_string(),
        })
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

fn execute_gpu(numbers: &[u32]) -> Option<Vec<u32>> {
    // Instantiates instance of WebGPU
    let gpu = Gpu::builder().build_headless().ok()?;

    // Instantiates buffer with data (`numbers`).
    let storage_buffer = gpu
        .new_buffer("Storage Buffer")
        .as_storage_buffer()
        .allow_copy_from()
        .allow_map_read()
        .create(numbers);

    // A bind group defines how buffers are accessed by shaders.
    let bind_group = gpu.create_bind_group(&[storage_buffer.bind_ssbo().in_compute()]);

    // A pipeline defines how shaders are executed.
    gpu.new_compute()
        .with_shader(include_bytes!("shader/hello-compute.wgsl"))
        .create_with_bindings(&[&bind_group])
        .dispatch(&[numbers.len() as u32]);

    // Read data from buffer. Staging buffer is created internally.
    let buffer_content = storage_buffer.download_immediately().ok()?;
    Some(bytemuck::cast_slice(buffer_content.deref()).to_owned())
}
