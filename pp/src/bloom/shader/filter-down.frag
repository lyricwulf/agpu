#version 460

layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D t;

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

struct SamplePoint {
    ivec2 offset;
    float weight;
};

// 36-texel downsample using 13 bilinear samples
// [Reference](http://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare)
// 0.25        0.125       0.125       0.125       0.125       1.0
// - - - - -   1 - 1 - -   - - 1 - 1   - - - - -   - - - - -   1 - 2 - 1
// - 4 - 4 -   - - - - -   - - - - -   - - - - -   - - - - -   - 4 - 4 -
// - - - - - + 1 - 1 - - + - - 1 - 1 + - - 1 - 1 + 1 - 1 - - = 2 - 4 - 2 
// - 4 - 4 -   - - - - -   - - - - -   - - - - -   - - - - -   - 4 - 4 -
// - - - - -   - - - - -   - - - - -   - - 1 - 1   1 - 1 - -   1 - 2 - 1
const SamplePoint kernel[13] = {
    {ivec2(-2,-2), 0.125 / 4}, {ivec2(0,-2), 0.25 / 4}, {ivec2(2,-2), 0.125 / 4}, 
    {ivec2(-1,-1), 0.5 / 4},  {ivec2(1,-1), 0.5 / 4},                    
    {ivec2(-2, 0), 0.25 / 4},  {ivec2(0,0), 0.5 / 4},   {ivec2(2,0), 0.25 / 4},   
    {ivec2(-1, 1), 0.5 / 4},  {ivec2(1,1), 0.5 / 4},                     
    {ivec2(-2, 2), 0.125 / 4}, {ivec2(0,2), 0.25 / 4}, {ivec2(2,2), 0.125 / 4}   
};

void main() {
    // Get image size for the root (0th) mip
    vec2 image_size = textureSize(sampler2D(t, s), 0);
    // dx and dy, the size of a texel in UV
    vec2 d = 1.0 / image_size;

    // 0,0          2,0
    // |-----|-----|
    // |     |     |
    // |-----X-----|  where X is our desired starting point
    // |     |     |  (we are sampling between pixels for bilinear samples)
    // |-----|-----|
    // 0,2          2,2
    vec2 target_pixel = vec2((image_size * in_uv)) ;
    target_pixel *= d.xy;

#define SAMPLE_KERNEL(x) textureOffset(sampler2D(t, s), target_pixel, kernel[x].offset) * kernel[x].weight
    vec4 color = SAMPLE_KERNEL(0) + SAMPLE_KERNEL(1) + SAMPLE_KERNEL(2) 
               + SAMPLE_KERNEL(3) + SAMPLE_KERNEL(4) 
               + SAMPLE_KERNEL(5) + SAMPLE_KERNEL(6) + SAMPLE_KERNEL(7) 
               + SAMPLE_KERNEL(8) + SAMPLE_KERNEL(9) 
               + SAMPLE_KERNEL(10) + SAMPLE_KERNEL(11) + SAMPLE_KERNEL(12);
#undef SAMPLE_KERNEL

    out_color = color;
}