struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t_input_1: texture_2d<f32>;
@group(0) @binding(2) var t_input_2: texture_2d<f32>;
@group(0) @binding(3) var<uniform> details: Details;

struct Details {
    mix_ratio: f32,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return (details.mix_ratio) * textureSample(t_input_1, s, in.uv) + 
        (1.0 - details.mix_ratio) * textureSample(t_input_2, s, in.uv);
}
 
