<div align="center">
    <a href="https://github.com/lyricwulf/agpu">
        <img src=".github/logo.png" alt="Logo" width="256" />
    </a>
    <h2 align="center"> Abstract GPU Project </h2>
    <p align="center"> 
        The easiest and most ergonomic GPU library 
        <br/>
        <a href="https://github.com/lyricwulf/agpu/tree/main/examples"> Explore examples » </a>
        <br/>
        <br/>
        <a href="https://github.com/lyricwulf/agpu/actions/workflows/rust.yml"><img src="https://github.com/lyricwulf/agpu/actions/workflows/rust.yml/badge.svg"></a>
        <a href="https://crates.io/crates/agpu"><img src="https://img.shields.io/crates/v/agpu"></a>
        <a href="https://docs.rs/agpu"><img src="https://img.shields.io/docsrs/agpu"></a>
    </p>
</div>

---

`agpu` is an abstraction library to the [wgpu](https://github.com/gfx-rs/wgpu) library, with the goal of providing a GPU framework for both small applications and large engines alike, with minimal boilerplate and maximum readability.

## Quick Start
To get started with a program that renders to the screen:
```rust
fn main() -> Result<(), agpu::BoxError> {
    let program = agpu::GpuProgram::builder().build()?;

    let example_pipeline = program.gpu.create_pipeline().build();

    program.run_draw(move |mut frame| {
        frame
            .render_pass("Example render pass")
            .with_pipeline(&example_pipeline)
            .begin()
            .draw_triangle();
    })
}
```
More examples are available in the examples folder. 

## Goals
- The easiest GPU library
- No loss of API functionality for underlying libraries
- Zero (ideal) runtime cost

### Non-goals
- Managed rendering engine
- Adhering strictly to WebGPU standard

## State
`agpu` is in a very early stage of development. It strives to be as stable as the underlying wgpu library, but some features will be incomplete or missing.

The current goal is to replicate all wgpu examples using minimal code. 

## Style
Builder-style API is used:
- Avoids boilerplate and struct hell
- Allows user to opt-in to functionality 
- Using sensible defaults, default constructors are one-liners
  
[`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) is **abused**([?](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html)) to add redundant/convenience functions to wgpu types. This is currently preferred to utility traits that add functions to the underlying types to avoid needing to include various traits that are not used directly.

## Integrations

Some integrations are provided as default features to this crate:
- [`winit`](https://github.com/rust-windowing/winit) for windowing (WIP)
- [`egui`](https://github.com/emilk/egui) for GUI (WIP)

You can (*not yet!*) disable them by opting out of default features, as well as create your own integration using this library.

---

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
