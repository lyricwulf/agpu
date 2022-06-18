//! Post processing implementation for agpu.
//! Some important limitations:
//! - Swapchain texture must be in BGRA format.
//! - Texture in BGRA format cannot be bound as storage texture (write in shader).
//! - Texture cannot be copied to another texture with a different format.

pub mod bloom;
// pub use Bloom;
