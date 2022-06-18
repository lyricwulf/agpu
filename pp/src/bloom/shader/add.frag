#version 460

layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D t;

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 outColor;

float max_comp(vec3 v) {
    return max(max(v.x, v.y), v.z);
}

void main() {
    vec4 sampled = texture(sampler2D(t, s), in_uv);

    outColor = sampled;
}