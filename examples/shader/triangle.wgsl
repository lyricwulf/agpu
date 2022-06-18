struct VertexInput {
  [[builtin(vertex_index)]] vertex_index: u32;
};

struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>;
  [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out : VertexOutput;
    let x = f32(i32(in.vertex_index) - 1)/2.0;
    let y = f32(i32(in.vertex_index & 1u) * 2 - 1)/2.0;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = vec4<f32>(f32(in.vertex_index == 1u), f32(in.vertex_index == 0u), f32(in.vertex_index == 2u), 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color / max(in.color.x, max(in.color.y, in.color.z));
}