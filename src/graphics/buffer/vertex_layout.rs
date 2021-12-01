pub use agpu_macro::{VertexLayout, VertexLayoutInstance};

use bytemuck::{Pod, Zeroable};

pub trait VertexLayout {
    fn vertex_buffer_layout<const L: u32>() -> wgpu::VertexBufferLayout<'static>;
}

pub trait VertexLayoutImpl {
    fn vertex_buffer_layout<const L: u32>() -> wgpu::VertexBufferLayout<'static>;
}

macro_rules! gen_norm_types {
    ($($norm:ident => $t:ty),*) => {
        $(
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Default, Zeroable, Pod, Debug)]
            #[repr(transparent)]
            pub struct $norm($t);
            impl std::ops::Deref for $norm {
                type Target = $t;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl std::ops::DerefMut for $norm {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        )*
    };
}

gen_norm_types!(
    i8n => i8,
    i16n => i16,
    u8n => u8,
    u16n => u16
);

/// Automatic vertex format derivation
/// Some general assumptions are made here:
/// - All fields are primitive types (or array, or matrix (TODO), or tuple)
/// - (TODO) Zero-sized fields will be ignored
/// ### Tuples vs Arrays
/// - For 8- and 16-bit integers:
///   - tuples will be passed to shader as a uvec or ivec
///   - arrays will be passed to shader as a normalized vec (converted to 0.0..1.0)
/// - For 32- and 64-bit integers, tuples/arrays are passed as uvec or ivec
/// - For floats, tuples/arrays are passed as vec
pub trait VertexFormatType {
    const VERTEX_FORMAT_TYPE: wgpu::VertexFormat;
}

macro_rules! gen_vertex_format_types {
    ($($t:ty => $v:ident),*) => {
        $(
            impl VertexFormatType for $t {
                const VERTEX_FORMAT_TYPE: ::wgpu::VertexFormat = ::wgpu::VertexFormat::$v;
            }
        )*
    };
}

// TODO: Support matrix input, creating multiple attributes for each matrix

gen_vertex_format_types!(
    [u8; 2] => Uint8x2,
    [u8; 4] => Uint8x4,
    [i8; 2] => Sint8x2,
    [i8; 4] => Sint8x4,
    [u8n; 2] => Unorm8x2,
    [u8n; 4] => Unorm8x4,
    [i8n; 2] => Snorm8x2,
    [i8n; 4] => Snorm8x4,
    [u16; 2] => Uint16x2,
    [u16; 4] => Uint16x4,
    [i16; 2] => Sint16x2,
    [i16; 4] => Sint16x4,
    [u16n; 2] => Unorm16x2,
    [u16n; 4] => Unorm16x4,
    [i16n; 2] => Snorm16x2,
    [i16n; 4] => Snorm16x4,
    f32 => Float32,
    [f32; 1] => Float32,
    [f32; 2] => Float32x2,
    [f32; 3] => Float32x3,
    [f32; 4] => Float32x4,
    u32 => Uint32,
    [u32; 1] => Uint32,
    [u32; 2] => Uint32x2,
    [u32; 3] => Uint32x3,
    [u32; 4] => Uint32x4,
    i32 => Sint32,
    [i32; 1] => Sint32,
    [i32; 2] => Sint32x2,
    [i32; 3] => Sint32x3,
    [i32; 4] => Sint32x4,
    f64 => Float64,
    [f64; 1] => Float64,
    [f64; 2] => Float64x2,
    [f64; 3] => Float64x3,
    [f64; 4] => Float64x4
);

// Tuples aren't Pod-derivable, so it is more convenient to use unorm type
gen_vertex_format_types!(
    (u8, u8) => Uint8x2,
    (u8, u8, u8, u8) => Uint8x4,
    (i8, i8) => Sint8x2,
    (i8, i8, i8, i8) => Sint8x4,
    (u16, u16) => Uint16x2,
    (u16, u16, u16, u16) => Uint16x4,
    (i16, i16) => Sint16x2,
    (i16, i16, i16, i16) => Sint16x4,
    (f32, f32) => Float32x2,
    (f32, f32, f32) => Float32x3,
    (f32, f32, f32, f32) => Float32x4,
    (u32, u32) => Uint32x2,
    (u32, u32, u32) => Uint32x3,
    (u32, u32, u32, u32) => Uint32x4,
    (i32, i32) => Sint32x2,
    (i32, i32, i32) => Sint32x3,
    (i32, i32, i32, i32) => Sint32x4,
    (f64, f64) => Float64x2,
    (f64, f64, f64) => Float64x3,
    (f64, f64, f64, f64) => Float64x4
);

// Vertex types for half floats, supported by half crate feature
#[cfg(feature = "half")]
gen_vertex_format_types!(
    [half::f16; 2] => Float16x2,
    [half::f16; 4] => Float16x4
);
