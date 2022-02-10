#![allow(clippy::module_name_repetitions)]

mod graphics;
pub use graphics::*;

mod compute;
pub use compute::*;

pub mod prelude;

/// Export wgpu crate
pub use wgpu;

/// Export half crate   
#[cfg(feature = "half")]
pub use half::{bf16, f16};

pub(crate) mod macros;

pub_const_flag!(QUERYSET_BUFFER_USAGE: wgpu::BufferUsages = MAP_READ | COPY_DST);

#[cfg(feature = "winit")]
pub mod winit;
pub use crate::winit::*;

#[cfg(feature = "egui")]
pub mod egui;
pub use crate::egui::*;
