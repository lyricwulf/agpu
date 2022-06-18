#version 460

layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D t;

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

struct SamplePoint {
    ivec2 offset;
    float weight;
};

// Progressive upsample using 3x3 tent filter
// [Reference](http://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare)
// 1 2 1
// 2 4 2
// 1 2 1
const SamplePoint kernel[9] = { 
    {ivec2(-1,-1), 1 / 16.0},  {ivec2(0,-1), 2 / 16.0}, {ivec2(1,-1), 1 / 16.0}, 
    {ivec2(-1, 0), 2 / 16.0},  {ivec2(0, 0), 4 / 16.0}, {ivec2(1, 0), 2 / 16.0}, 
    {ivec2(-1, 1), 1 / 16.0},  {ivec2(0, 1), 2 / 16.0}, {ivec2(1, 1), 1 / 16.0},                      
};

void main() {
    // Get image size
    vec2 image_size = textureSize(sampler2D(t, s), 0);
    // dx and dy, the size of a texel in UV
    vec2 d = 1.0 / image_size;
    vec2 radius = vec2(1.0);

    vec2 target_pixel = vec2((image_size * in_uv)) + 1;
    target_pixel *= d.xy;

#define SAMPLE_KERNEL(x) texture(sampler2D(t, s), target_pixel + kernel[x].offset * d * radius) * kernel[x].weight
    vec4 color = SAMPLE_KERNEL(0) + SAMPLE_KERNEL(1) + SAMPLE_KERNEL(2) 
               + SAMPLE_KERNEL(3) + SAMPLE_KERNEL(4) + SAMPLE_KERNEL(5)
               + SAMPLE_KERNEL(6) + SAMPLE_KERNEL(7) + SAMPLE_KERNEL(8);
#undef SAMPLE_KERNEL

    if (color.r == color.g  && color.r == color.b) {
        color = vec4(0);
    }
    // threshold
    color = smoothstep(0.0, 1.0, color);
    // output
    out_color = color;
    
}