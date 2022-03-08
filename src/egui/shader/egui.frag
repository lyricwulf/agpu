#version 460

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_uv;
layout (location = 2) in vec4 in_color;

// layout (set = 0, binding = 1) uniform texture2D tex;
// layout (set = 0, binding = 2) uniform sampler s;

layout (location = 0) out vec4 out_color;

void main() {
    out_color = in_color ;
}
