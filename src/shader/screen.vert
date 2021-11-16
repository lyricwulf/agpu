//! Vertex shader that draws one triangle which spans the entire screen.
//! Top left is (0, 0). 
//! Inspired by https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
//! This has been changed to use consts instead of calculations of `gl_VertexIndex`.

#version 460

layout (location = 0) out vec2 out_uv;

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 pos[3] = vec2[](vec2(-1, 1), vec2(3, 1), vec2(-1, -3));
const vec2 uv[3] = vec2[](vec2(0, 0), vec2(2, 0), vec2(0, 2));

void main() {
    out_uv = uv[gl_VertexIndex];
    gl_Position = vec4(pos[gl_VertexIndex], 0, 1); 
}