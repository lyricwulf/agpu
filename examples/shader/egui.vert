//! Vertex shader that simply passes the gl_Position from the given vertex

#version 460

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_uv;
layout (location = 2) in vec4 in_color;

layout (set = 0, binding = 0) uniform Viewport {
    vec2 size;
} viewport;

layout (location = 0) out vec2 out_pos;
layout (location = 1) out vec2 out_uv;
layout (location = 2) out vec4 out_color;

void main() {
    vec2 xy = (in_pos / viewport.size) * 2 - 1;
    gl_Position = vec4(xy, 0, 1); 
    gl_Position.y = - gl_Position.y;
    out_pos = in_pos;
    out_uv = in_uv;
    out_color = in_color;
}
